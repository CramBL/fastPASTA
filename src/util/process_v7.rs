type V7 = crate::words::rdh::RdhCRUv7;
type Payload = Vec<Vec<u8>>;
type Cdp = (Vec<V7>, Payload);
use crossbeam_channel::{bounded, Receiver};
use std::sync::atomic::{AtomicBool, Ordering};
// Larger capacity means less overhead, but more memory usage
// Too small capacity will cause the producer thread to block
// Too large capacity will cause down stream consumers to block
const CHANNEL_CDP_CAPACITY: usize = 100;

pub mod input {
    use super::*;
    use crate::get_chunk;
    use crate::util::bufreader_wrapper::BufferedReaderWrapper;
    use crate::util::input_scanner::InputScanner;
    use crate::words::rdh::RDH;

    type Payload = Vec<Vec<u8>>;

    pub fn spawn_reader<T: RDH + 'static>(
        stop_flag: std::sync::Arc<AtomicBool>,
        input_scanner: InputScanner<
            impl BufferedReaderWrapper + ?Sized + std::marker::Send + 'static,
        >,
    ) -> (std::thread::JoinHandle<()>, Receiver<(Vec<T>, Payload)>) {
        let reader_thread = std::thread::Builder::new().name("Reader".to_string());
        let (send_channel, rcv_channel) = bounded(CHANNEL_CDP_CAPACITY);
        let thread_handle = reader_thread
            .spawn({
                move || {
                    let mut input_scanner = input_scanner;

                    // Automatically extracts link to filter if one is supplied
                    loop {
                        if stop_flag.load(Ordering::SeqCst) {
                            log::trace!("Stopping reader thread");
                            break;
                        }
                        let (rdh_chunk, payload_chunk) =
                            match get_chunk::<T>(&mut input_scanner, 100) {
                                Ok(cdp) => cdp,
                                Err(e) => {
                                    if e.kind() == std::io::ErrorKind::UnexpectedEof {
                                        break;
                                    } else {
                                        panic!("Error reading CDP chunks: {e}");
                                    }
                                }
                            };

                        // Send a chunk to the checker
                        if let Err(e) = send_channel.try_send((rdh_chunk, payload_chunk)) {
                            if e.is_full() {
                                log::trace!("Checker is too slow");
                                if send_channel.send(e.into_inner()).is_err()
                                    && !stop_flag.load(Ordering::SeqCst)
                                {
                                    log::trace!("Unexpected error while sending data to checker");
                                    break;
                                }
                            } else if stop_flag.load(Ordering::SeqCst) {
                                log::trace!("Stopping reader thread");
                                break;
                            }
                        }
                    }
                }
            })
            .expect("Failed to spawn reader thread");
        (thread_handle, rcv_channel)
    }
}

pub mod validate {
    use super::*;

    use crate::util::config::Opt;
    use crate::util::stats::StatType;
    use crate::validators::cdp_running::CdpRunningValidator;
    use crate::validators::rdh::RdhCruv7RunningChecker;
    use crate::words::rdh::{RdhCRUv7, RDH};
    use crossbeam_channel::{bounded, Receiver, RecvError};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{mpsc, Arc};
    use std::thread::JoinHandle;
    // Larger capacity means less overhead, but more memory usage
    // Too small capacity will cause the checker thread to block
    // Too large capacity will cause down stream consumers to block

    #[inline]
    pub fn spawn_checker(
        config: Arc<Opt>,
        stop_flag: Arc<AtomicBool>,
        stats_sender_channel: mpsc::Sender<StatType>,
        data_channel: Receiver<Cdp>,
    ) -> (JoinHandle<()>, Option<Receiver<Cdp>>) {
        let checker_thread = std::thread::Builder::new().name("Checker".to_string());
        let (send_channel, rcv_channel): (crossbeam_channel::Sender<Cdp>, Receiver<Cdp>) =
            bounded(CHANNEL_CDP_CAPACITY);
        let validator_handle = checker_thread
            .spawn({
                let config = config.clone();
                let mut current_mem_pos: u64 = 0;
                move || {
                    let mut cdp_payload_running_validator =
                        CdpRunningValidator::new(stats_sender_channel.clone());
                    let mut running_rdh_checker = RdhCruv7RunningChecker::new();

                    while !stop_flag.load(Ordering::SeqCst) {
                        // Receive chunk from reader
                        let (rdh_chunk, payload_chunk) = match data_channel.recv() {
                            Ok(cdp) => cdp,
                            Err(e) => {
                                debug_assert_eq!(e, RecvError);
                                break;
                            }
                        };

                        if config.any_checks() {
                            do_checks_v7(
                                &rdh_chunk,
                                &payload_chunk,
                                &stats_sender_channel,
                                &mut running_rdh_checker,
                                &mut cdp_payload_running_validator,
                                current_mem_pos,
                            );
                        }

                        // Increment the current memory position based on RDH and payload sizes
                        current_mem_pos += rdh_chunk.len() as u64 * 512;
                        payload_chunk.iter().for_each(|v| {
                            current_mem_pos += v.len() as u64 * 8;
                        });

                        // Send chunk to the checker
                        match config.output_mode() {
                            crate::util::config::DataOutputMode::None => {} // Do nothing
                            _ => {
                                if send_channel.send((rdh_chunk, payload_chunk)).is_err()
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
        rdh_slices: &[RdhCRUv7],
        payload_slices: &[Vec<u8>],
        stats_sender_ch_checker: &std::sync::mpsc::Sender<StatType>,
        rdh_running: &mut RdhCruv7RunningChecker,
        payload_running: &mut CdpRunningValidator<RdhCRUv7>,
        current_mem_pos: u64,
    ) {
        let mut tmp_mem_pos_tracker = current_mem_pos;
        for (rdh, payload) in rdh_slices.iter().zip(payload_slices.iter()) {
            do_rdh_checks(rdh, rdh_running, stats_sender_ch_checker);
            tmp_mem_pos_tracker += 512;
            payload_running.set_current_rdh(rdh, tmp_mem_pos_tracker);
            do_payload_checks(
                payload,
                rdh.data_format(),
                payload_running,
                stats_sender_ch_checker,
            );
            tmp_mem_pos_tracker += payload.len() as u64 * 8;
        }
    }

    #[inline]
    fn do_rdh_checks(
        rdh: &V7,
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
        rdh: &V7,
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

pub mod output {
    use super::*;

    use crate::util::{config::Opt, writer::BufferedWriter, writer::Writer};
    use std::{
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
        thread,
    };
    const BUFFER_SIZE: usize = 1024 * 1024; // 1MB buffer
    pub fn spawn_writer(
        config: Arc<Opt>,
        stop_flag: Arc<AtomicBool>,
        data_channel: Receiver<Cdp>,
    ) -> thread::JoinHandle<()> {
        let writer_thread = thread::Builder::new().name("Writer".to_string());
        writer_thread
            .spawn({
                let mut writer = BufferedWriter::<V7>::new(&config, BUFFER_SIZE);
                move || loop {
                    // Receive chunk from checker
                    let (rdh_chunk, payload_chunk) = match data_channel.recv() {
                        Ok(cdp) => cdp,
                        Err(e) => {
                            debug_assert_eq!(e, crossbeam_channel::RecvError);
                            break;
                        }
                    };
                    if stop_flag.load(Ordering::SeqCst) {
                        log::trace!("Stopping writer thread");
                        break;
                    }
                    // Push data onto the writer's buffer, which will flush it when the buffer is full or when the writer is dropped
                    writer.push_cdps_raw((rdh_chunk, payload_chunk));
                }
            })
            .expect("Failed to spawn writer thread")
    }
}
