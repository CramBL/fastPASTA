use super::cdp_running::CdpRunningValidator;
use super::rdh::RdhCruSanityValidator;
use super::rdh_running::RdhCruRunningChecker;
use crate::input::data_wrapper::CdpChunk;
use crate::stats::stats_controller::StatType;
use crate::util::lib::Config;
use crate::words::lib::RDH;
use crate::words::lib::{layer_from_feeid, stave_number_from_feeid};
use crossbeam_channel::{bounded, Receiver, RecvError};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread::JoinHandle;

#[inline]
pub fn spawn_validator<T: RDH + 'static>(
    config: Arc<impl Config + 'static>,
    stop_flag: Arc<AtomicBool>,
    stats_sender_channel: mpsc::Sender<StatType>,
    data_channel: Receiver<CdpChunk<T>>,
) -> (JoinHandle<()>, Option<Receiver<CdpChunk<T>>>) {
    let validator_thread = std::thread::Builder::new().name("Validator".to_string());
    let (send_channel, rcv_channel) = bounded(crate::CHANNEL_CDP_CAPACITY);
    let validator_handle = validator_thread
        .spawn({
            let config = config.clone();
            move || {
                let mut cdp_payload_running_validator =
                    CdpRunningValidator::new(&*config, stats_sender_channel.clone());
                let mut running_rdh_checker = RdhCruRunningChecker::new();
                let mut sanity_rdh_checker = RdhCruSanityValidator::new();

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
                    // Send HBF seen if stop bit is 1
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

                    if config.check().is_some() {
                        do_checks(
                            &*config,
                            &cdp_chunk,
                            &stats_sender_channel,
                            &mut sanity_rdh_checker,
                            &mut running_rdh_checker,
                            &mut cdp_payload_running_validator,
                        );
                    }

                    // Send chunk to the checker
                    match config.output_mode() {
                        crate::util::lib::DataOutputMode::None => {} // Do nothing
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
        crate::util::lib::DataOutputMode::None => (validator_handle, None),
        _ => (validator_handle, Some(rcv_channel)),
    }
}

#[inline]
fn do_checks<T: RDH>(
    config: &impl Config,
    cdp_chunk: &CdpChunk<T>,
    stats_sender_ch_checker: &std::sync::mpsc::Sender<StatType>,
    rdh_sanity: &mut RdhCruSanityValidator<T>,
    rdh_running: &mut RdhCruRunningChecker<T>,
    payload_running: &mut CdpRunningValidator<T>,
) {
    cdp_chunk
        .into_iter()
        .enumerate()
        .for_each(|(rdh_idx, (rdh, payload, rdh_mem_pos))| {
            stats_sender_ch_checker
                .send(StatType::DataFormat(rdh.data_format()))
                .unwrap();
            // 1. RDH sanity and running checks
            if let Err(mut e) = rdh_checks::do_rdh_checks(config, rdh, rdh_sanity, rdh_running) {
                e.push_str(crate::words::rdh_cru::RdhCRU::<crate::words::rdh_cru::V7>::rdh_header_text_with_indent_to_string(7).as_str());
                let rdhs = cdp_chunk.rdh_slice();
                match rdh_idx {
                    0 => log::warn!("Error occured in the first RDH in a CdpChunk, it is not possible to retrieve previous RDHS"),
                    1 => {
                        log::warn!("Error occured in the second RDH in a CdpChunk, can only retrieve 1 previous RDH");
                        e.push_str(&format!("{}", rdhs.first().unwrap()));
                        e.push_str(&format!("\n{rdh}"));
                        e.push_str("<--- Error occured here\n");
                        e.push_str(&format!("{}", rdhs.get(rdh_idx + 1).unwrap()));
                    }
                    // Last RDH of the CDP Chunk
                    _ if rdhs.len() == (rdh_idx + 1) => {
                        log::warn!("Error occured in last RDH in CdpChunk, it is not possible to retrieve the next RDH");
                        e.push_str(&format!("{}\n", rdhs.get(rdh_idx - 2).unwrap()));
                        e.push_str(&format!("{}", rdhs.get(rdh_idx - 1).unwrap()));
                        e.push_str(&format!("\n{rdh}"));
                        e.push_str("<--- Error occured here\n");
                    }
                    _ => {
                        e.push_str(&format!("{}\n", rdhs.get(rdh_idx - 2).unwrap()));
                        e.push_str(&format!("{}", rdhs.get(rdh_idx - 1).unwrap()));
                        e.push_str(&format!("\n{rdh}"));
                        e.push_str("<--- Error occured here\n");
                        e.push_str(&format!("{}", rdhs.get(rdh_idx + 1).unwrap()));
                    }
                }
                stats_sender_ch_checker
                    .send(StatType::Error(format!("{rdh_mem_pos:#X}: {e}")))
                    .unwrap();
            }

            // 2. Payload sanity and running checks
            use crate::util::lib::Check;
            if let Some(check) = config.check() {
                match check {
                    // Only check if the target is not set (i.e. check all)
                    Check::All(_) | Check::Sanity(_) | Check::Running(_) if check.target().is_none() => {
                        payload_running.set_current_rdh(rdh, rdh_mem_pos);
                        if !payload.is_empty() {
                            payload_checks::do_payload_checks(
                                payload,
                                rdh.data_format(),
                                payload_running,
                                stats_sender_ch_checker,
                            );
                        } else {
                            log::debug!("Empty payload at {:#X}", rdh_mem_pos + 64);
                        }
                }
                    _ => {}
                }
            }
        });
}

mod rdh_checks {
    use crate::util::lib::{Config, Data};
    use crate::validators::rdh::RdhCruSanityValidator;
    use crate::validators::rdh_running::RdhCruRunningChecker;
    use crate::words::lib::RDH;

    #[inline]
    pub fn do_rdh_checks<T: RDH>(
        config: &impl Config,
        rdh: &T,
        sanity_rdh_checker: &mut RdhCruSanityValidator<T>,
        running_rdh_checker: &mut RdhCruRunningChecker<T>,
    ) -> Result<(), String> {
        // Check if any checks have been specified in the config
        if let Some(check) = config.check() {
            use crate::util::lib::Check;
            // Check if the check is for the current target (rdh)
            //  then check if all checks are to be performed or just a specific one
            match check {
                Check::All(_) => match check.target() {
                    // If target is Rdh or None (all targets) then we check
                    Some(Data::Rdh) | None => {
                        sanity_rdh_checker.sanity_check(rdh)?;
                        running_rdh_checker.check(rdh)?;
                    }
                },
                Check::Sanity(_) => match check.target() {
                    // If target is Rdh or None (all targets) then we check
                    Some(Data::Rdh) | None => {
                        sanity_rdh_checker.sanity_check(rdh)?;
                    }
                },
                Check::Running(_) => match check.target() {
                    // If target is Rdh or None (all targets) then we check
                    Some(Data::Rdh) | None => {
                        running_rdh_checker.check(rdh)?;
                    }
                },
            }
        }

        Ok(())
    }
}

mod payload_checks {
    use crate::words::lib::RDH;
    use crate::{stats::stats_controller::StatType, validators::cdp_running::CdpRunningValidator};

    #[inline]
    pub fn do_payload_checks<T: RDH>(
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
}
