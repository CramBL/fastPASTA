use crate::input::data_wrapper::CdpChunk;
use crate::stats::stats_controller::StatType;
use crate::util::config::Opt;
use crate::words::rdh::{layer_from_feeid, stave_number_from_feeid, RdhCRUv7, RDH};
use crossbeam_channel::{bounded, Receiver, RecvError};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread::JoinHandle;

use super::cdp_running::CdpRunningValidator;
use super::rdh::RdhCruv7RunningChecker;

type RdhV7 = crate::words::rdh::RdhCRUv7;

#[inline]
pub fn spawn_checker(
    config: Arc<Opt>,
    stop_flag: Arc<AtomicBool>,
    stats_sender_channel: mpsc::Sender<StatType>,
    data_channel: Receiver<CdpChunk<RdhV7>>,
) -> (JoinHandle<()>, Option<Receiver<CdpChunk<RdhV7>>>) {
    let checker_thread = std::thread::Builder::new().name("Checker".to_string());
    let (send_channel, rcv_channel) = bounded(crate::CHANNEL_CDP_CAPACITY);
    let validator_handle = checker_thread
        .spawn({
            let config = config.clone();
            move || {
                let mut cdp_payload_running_validator =
                    CdpRunningValidator::new(stats_sender_channel.clone());
                let mut running_rdh_checker = RdhCruv7RunningChecker::new();

                while !stop_flag.load(Ordering::SeqCst) {
                    // Receive chunk from reader
                    let cdp_chunk = match data_channel.recv() {
                        Ok(cdp) => cdp,
                        Err(e) => {
                            debug_assert_eq!(e, RecvError);
                            break;
                        }
                    };
                    // Collect global stats
                    // Send HBF seen if HBA is detected
                    cdp_chunk.rdh_slice().iter().for_each(|rdh| {
                        if rdh.stop_bit() == 1 {
                            stats_sender_channel.send(StatType::HBFsSeen(1)).unwrap();
                        }
                        let layer = layer_from_feeid(rdh.fee_id());
                        let stave = stave_number_from_feeid(rdh.fee_id());
                        stats_sender_channel
                            .send(StatType::LayerStaveSeen { layer, stave })
                            .unwrap();
                    });

                    if config.any_checks() {
                        do_checks_v7(
                            &cdp_chunk,
                            &stats_sender_channel,
                            &mut running_rdh_checker,
                            &mut cdp_payload_running_validator,
                        );
                    }

                    // Send chunk to the checker
                    match config.output_mode() {
                        crate::util::config::DataOutputMode::None => {} // Do nothing
                        _ => {
                            if send_channel.send(cdp_chunk).is_err()
                                && !stop_flag.load(Ordering::SeqCst)
                            {
                                log::trace!("Unexpected error while sending data to writer");
                                break;
                            }
                        }
                    }
                }
            }
        })
        .expect("Failed to spawn checker thread");

    match config.output_mode() {
        crate::util::config::DataOutputMode::None => (validator_handle, None),
        _ => (validator_handle, Some(rcv_channel)),
    }
}

#[inline]
fn do_checks_v7(
    cdp_chunk: &CdpChunk<RdhV7>,
    stats_sender_ch_checker: &std::sync::mpsc::Sender<StatType>,
    rdh_running: &mut RdhCruv7RunningChecker,
    payload_running: &mut CdpRunningValidator<RdhCRUv7>,
) {
    cdp_chunk.into_iter().for_each(|(rdh, payload, mem_pos)| {
        stats_sender_ch_checker
            .send(StatType::DataFormat(rdh.data_format()))
            .unwrap();

        do_rdh_checks(rdh, rdh_running, stats_sender_ch_checker);

        payload_running.set_current_rdh(rdh, mem_pos);
        if !payload.is_empty() {
            do_payload_checks(
                payload,
                rdh.data_format(),
                payload_running,
                stats_sender_ch_checker,
            );
        } else {
            log::debug!("Empty payload at {mem_pos}");
        }
    });
}

#[inline]
fn do_rdh_checks(
    rdh: &RdhV7,
    running_rdh_checker: &mut RdhCruv7RunningChecker,
    stats_sender_ch_checker: &std::sync::mpsc::Sender<StatType>,
) {
    do_rdh_v7_running_checks(rdh, running_rdh_checker, stats_sender_ch_checker);
}

#[inline]
fn do_payload_checks<T: RDH>(
    payload: &[u8],
    dataformat: u8,
    payload_running: &mut CdpRunningValidator<T>,
    stats_sender_ch_checker: &std::sync::mpsc::Sender<StatType>,
) {
    match preprocess_payload(payload, dataformat) {
        Ok(gbt_word_chunks) => gbt_word_chunks.for_each(|gbt_word| {
            payload_running.check(&gbt_word[..10]); // Take 10 bytes as flavor 0 would have additional 6 bytes of padding
        }),
        Err(e) => {
            stats_sender_ch_checker.send(StatType::Error(e)).unwrap();
            payload_running.reset_fsm();
        }
    }
}

#[inline]
fn do_rdh_v7_running_checks(
    rdh: &RdhV7,
    running_rdh_checker: &mut RdhCruv7RunningChecker,
    stats_sender_ch_checker: &std::sync::mpsc::Sender<StatType>,
) {
    // RDH CHECK: There is always page 0 + minimum page 1 + stop flag
    if let Err(e) = running_rdh_checker.check(rdh) {
        stats_sender_ch_checker
            .send(StatType::Error(format!("RDH check failed: {e}")))
            .unwrap();
    }
}

#[inline]
fn preprocess_payload(
    payload: &[u8],
    dataformat: u8,
) -> Result<impl Iterator<Item = &[u8]>, String> {
    // Check payload size
    log::trace!("Payload size is {}", payload.len());

    // Retrieve end of payload padding from payload
    let ff_padding = payload
        .iter()
        .rev()
        .take_while(|&x| *x == 0xFF)
        .collect::<Vec<_>>();

    // Exceeds the maximum padding of 15 bytes that is required to pad to 16 bytes
    if ff_padding.len() > 15 {
        return Err(format!("End of payload 0xFF padding is {} bytes, exceeding max of 15 bytes: Skipping current payload",
        ff_padding.len()));
    }

    // Determine if padding is flavor 0 (6 bytes of 0x00 padding following GBT words) or flavor 1 (no padding)
    // Using an iterator approach instead of indexing also supports the case where the payload is smaller than 16 bytes or even empty
    let detected_data_format = if payload
        .iter() // Create an iterator over the payload
        .take(16) // Take the first 16 bytes
        .rev() // Now reverse the iterator
        .take_while(|&x| *x == 0x00) // Take bytes while they are equal to 0x00
        .count() // Count them and check if they are equal to 6
        == 6
    {
        log::trace!("Data format 0 detected");
        0
    } else {
        log::trace!("Data format 2 detected");
        2
    };

    // Split payload into GBT words sized slices, using chunks_exact to allow more compiler optimizations
    let gbt_word_chunks = if detected_data_format == 0 {
        // If flavor 0, dividing into 16 byte chunks should cut the payload up with no remainder
        let chunks = payload.chunks_exact(16);
        debug_assert!(chunks.remainder().is_empty());
        debug_assert!(dataformat == 0);
        chunks
    }
    // If flavor 1, and the padding is more than 9 bytes, padding will be processed as a GBT word, therefor exclude it from the slice
    //    Before calling chunks_exact
    else if ff_padding.len() > 9 {
        let last_idx_before_padding = payload.len() - ff_padding.len();
        let chunks = payload[..last_idx_before_padding].chunks_exact(10);
        debug_assert!(chunks.remainder().is_empty());
        debug_assert!(dataformat == 2);
        chunks
    } else {
        // Simply divide into 10 byte chunks and assert that the remainder is padding bytes
        let chunks = payload.chunks_exact(10);
        debug_assert!(chunks.remainder().iter().all(|&x| x == 0xFF)); // Asserts that the payload padding is 0xFF
        debug_assert!(dataformat == 2);
        chunks
    };

    Ok(gbt_word_chunks)
}
