#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_code)]
use crossbeam_channel::{bounded, Receiver, RecvError, Sender};
use fastpasta::util::bufreader_wrapper::BufferedReaderWrapper;
use fastpasta::util::config::Opt;
use fastpasta::util::file_pos_tracker::FilePosTracker;
use fastpasta::util::file_scanner::{FileScanner, ScanCDP};
use fastpasta::util::stats::{self, StatType};
use fastpasta::util::writer::{BufferedWriter, Writer};
use fastpasta::validators::cdp_running::CdpRunningValidator;
use fastpasta::validators::rdh::RdhCruv7RunningChecker;
use fastpasta::words::rdh::RDH;
use fastpasta::words::rdh::{Rdh0, RdhCRUv6, RdhCRUv7};
use fastpasta::{init_stats_controller, ByteSlice, GbtWord};
use log::{debug, info, trace};
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, vec};
use structopt::StructOpt;

fn init_error_logger(cfg: &Opt) {
    stderrlog::new()
        .module(module_path!())
        .verbosity(cfg.verbosity() as usize)
        .init()
        .expect("Failed to initialize logger");
}

pub fn main() {
    let opt: Opt = StructOpt::from_args();
    trace!("{:#?}", opt);
    let config: Arc<Opt> = Arc::new(opt);

    init_error_logger(&config.clone());
    // let config: Opt = <Opt as structopt::StructOpt>::from_iter(&[
    //     "fastpasta",
    //     "-s",
    //     "-f",
    //     "0",
    //     "../fastpasta_test_files/data_ols_ul.raw",
    //     "-o test_filter_link.raw",
    // ]);

    {
        // Launch statistics thread
        // If max allowed errors is reached, stop the processing from the stats thread
        let (stat_controller, stat_send_channel, stop_flag) =
            init_stats_controller(&config.clone());

        let mut readable = fastpasta::init_reader(&config.clone());

        // Determine RDH version
        let rdh0 = Rdh0::load(&mut readable).expect("Failed to read first RDH0");
        let rdh_version = rdh0.header_id;
        let loader = FileScanner::new_from_rdh0(
            config.clone(),
            readable,
            FilePosTracker::new(),
            stat_send_channel.clone(),
            rdh0,
        );

        // Choose the rest of the execution based on the RDH version
        // Necessary to prevent heap allocation and allow static dispatch as the type cannot be known at compile time
        match rdh_version {
            6 => process_rdh_v6(config, loader, stat_send_channel, stop_flag.clone()).unwrap(),
            7 => process_rdh_v7(config, loader, stat_send_channel, stop_flag.clone()).unwrap(),
            _ => panic!("Unknown RDH version: {}", rdh_version),
        }
        stat_controller.join().expect("Failed to join stats thread");
    }
}

// 1. Setup reading (file or stdin) // TODO: stdin support
// 2. Do checks on read data
// 3. Write data out (file or stdout)
pub fn process_rdh_v7(
    config: Arc<Opt>,
    loader: FileScanner<impl BufferedReaderWrapper + ?Sized + std::marker::Send + 'static>,
    send_stats_ch: std::sync::mpsc::Sender<stats::StatType>,
    thread_stopper: Arc<AtomicBool>,
) -> io::Result<()> {
    // Types specific for RDH v7
    type V7 = RdhCRUv7;
    type Cdp = (Vec<V7>, Vec<Vec<u8>>);
    // Create producer-consumer channel for the reader to the checker
    let (sender_reader, receiver_checker): (Sender<Cdp>, Receiver<Cdp>) = bounded(100);
    // Create producer-consumer channel for the checker to the writer
    let (sender_checker, receiver_writer): (Sender<Cdp>, Receiver<Cdp>) = bounded(100);

    // Setup reader, checker, writer, stats
    let mut running_rdh_checker = RdhCruv7RunningChecker::new();
    let mut writer = BufferedWriter::<V7>::new(&config, 1024 * 1024); // 1MB buffer

    // 1. Read data from file
    let cfg = Arc::clone(&config);
    let stats_sender_ch_reader = send_stats_ch.clone();
    let reader_thread = thread::spawn({
        let stop_flag = thread_stopper.clone();
        move || {
            // Automatically extracts link to filter if one is supplied
            let mut file_scanner = loader;

            loop {
                if stop_flag.load(Ordering::SeqCst) {
                    trace!("Stopping reader thread");
                    break;
                }
                let (rdh_chunk, payload_chunk) = match get_chunk::<V7>(&mut file_scanner, 100) {
                    Ok(cdp) => cdp,
                    Err(e) => {
                        if e.kind() == io::ErrorKind::UnexpectedEof {
                            break;
                        } else {
                            panic!("Error reading CDP chunks: {}", e);
                        }
                    }
                };

                // Send a chunk to the checker
                if let Err(e) = sender_reader.try_send((rdh_chunk, payload_chunk)) {
                    if e.is_full() {
                        log::trace!("Checker is too slow");
                        if let Err(_) = sender_reader.send(e.into_inner()) {
                            if stop_flag.load(Ordering::SeqCst) == false {
                                log::trace!("Unexpected error while sending data to checker");
                                break;
                            }
                        }
                    } else {
                        if stop_flag.load(Ordering::SeqCst) {
                            log::trace!("Stopping reader thread");
                            break;
                        }
                    }
                }
            }
        }
    });

    // 2. Do checks on a received chunk of data

    let checker_thread = thread::spawn({
        let stop_flag = thread_stopper.clone();
        let cfg = config.clone();
        let stats_sender_ch_checker = send_stats_ch.clone();
        move || {
            let stats_sender_ch_validator = stats_sender_ch_checker.clone();
            let mut cdp_payload_running_validator =
                CdpRunningValidator::new(stats_sender_ch_validator);

            loop {
                // Receive chunk from reader
                let (rdh_chunk, payload_chunk) = match receiver_checker.recv() {
                    Ok(cdp) => cdp,
                    Err(e) => {
                        debug_assert_eq!(e, RecvError);
                        break;
                    }
                };
                if stop_flag.load(Ordering::SeqCst) {
                    trace!("Stopping checker thread");
                    break;
                }

                // Do checks one each pair of RDH and payload in the chunk
                for (rdh, payload) in rdh_chunk.iter().zip(payload_chunk.iter()) {
                    info!("{rdh}");
                    // Check RDH
                    if cfg.sanity_checks() {
                        if let Err(e) = sanity_validation(&rdh) {
                            stats_sender_ch_checker
                                .send(stats::StatType::Error(format!(
                                    "Sanity check failed: {}",
                                    e
                                )))
                                .unwrap();
                        }
                    }
                    do_rdh_v7_running_checks(
                        &rdh,
                        &mut running_rdh_checker,
                        &stats_sender_ch_checker,
                    );

                    // Check padding:
                    //  - Flavor 0 will have no padding - other than the usual 6 bytes with 0x0
                    //  - Flavor 1 will have padding if last word does not fill all 16 bytes
                    // Asserts that the payload is padded to 16 bytes at the end (Fails for data_ols_ul.raw as it is old from when the padding logic was bugged)
                    cdp_payload_running_validator.current_rdh =
                        Some(RdhCRUv7::load(&mut rdh.to_byte_slice()).unwrap());
                    match preprocess_payload(payload) {
                        Ok(gbt_word_chunks) => {
                            gbt_word_chunks.for_each(|gbt_word| {
                                if let Err(e) = cdp_payload_running_validator.check(rdh, gbt_word) {
                                    stats_sender_ch_checker
                                        .send(StatType::Error(format!(
                                            "Payload check failed for: {:?} - With error:{}",
                                            gbt_word, e
                                        )))
                                        .unwrap();
                                }
                            });
                        }
                        Err(e) => {
                            stats_sender_ch_checker.send(StatType::Error(e)).unwrap();
                            cdp_payload_running_validator.reset_fsm();
                        }
                    }
                }

                // Checks are done, send chunk to writer
                if let Err(_) = sender_checker.send((rdh_chunk, payload_chunk)) {
                    if stop_flag.load(Ordering::SeqCst) == false {
                        log::trace!("Unexpected error while sending data to writer");
                        break;
                    }
                }
            }
        }
    });

    // 3. Write data out
    let writer_thread = thread::spawn({
        let stop_flag = thread_stopper.clone();
        move || loop {
            // Receive chunk from checker
            let (rdh_chunk, payload_chunk) = match receiver_writer.recv() {
                Ok(cdp) => cdp,
                Err(e) => {
                    debug_assert_eq!(e, RecvError);
                    break;
                }
            };
            if stop_flag.load(Ordering::SeqCst) {
                trace!("Stopping writer thread");
                break;
            }
            // Push data onto the writer's buffer, which will flush it when the buffer is full or when the writer is dropped
            writer.push_cdps_raw((rdh_chunk, payload_chunk));
        }
    });

    reader_thread.join().expect("Error joining reader thread");
    checker_thread.join().expect("Error joining checker thread");
    writer_thread.join().expect("Error joining writer thread");
    Ok(())
}

pub fn process_rdh_v6(
    config: Arc<Opt>,
    loader: FileScanner<impl BufferedReaderWrapper + ?Sized>,
    send_stats_ch: std::sync::mpsc::Sender<stats::StatType>,
    thread_stopper: Arc<AtomicBool>,
) -> io::Result<()> {
    todo!("RDH v6 not implemented yet");
    // Automatically extracts link to filter if one is supplied
    let mut file_scanner = loader;

    let (rdh_chunk, _payload_chunk) =
        get_chunk::<RdhCRUv6>(&mut file_scanner, 10).expect("Error reading CDP chunks");

    for _rdh in rdh_chunk {
        if config.sanity_checks() {
            todo!("Sanity check for RDH v6")
        }
    }

    Ok(())
}

pub fn sanity_validation(rdh: &RdhCRUv7) -> Result<(), fastpasta::validators::rdh::GbtError> {
    let rdh_validator = fastpasta::validators::rdh::RDH_CRU_V7_VALIDATOR;
    rdh_validator.sanity_check(&rdh)
}

pub fn get_chunk<T: RDH>(
    file_scanner: &mut FileScanner<impl BufferedReaderWrapper + ?Sized>,
    chunk_size_cdps: usize,
) -> Result<(Vec<T>, Vec<Vec<u8>>), io::Error> {
    let mut rdhs: Vec<T> = vec![];
    let mut payloads: Vec<Vec<u8>> = vec![];

    for _ in 0..chunk_size_cdps {
        let (rdh, payload) = match file_scanner.load_cdp() {
            Ok(cdp) => cdp,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                info!("EOF reached! ");
                break;
            }
            Err(e) => return Err(e),
        };
        rdhs.push(rdh);
        payloads.push(payload);
    }

    if rdhs.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "No CDPs found",
        ));
    }

    Ok((rdhs, payloads))
}

pub fn do_rdh_v7_running_checks(
    rdh: &RdhCRUv7,
    running_rdh_checker: &mut RdhCruv7RunningChecker,
    stats_sender_ch_checker: &std::sync::mpsc::Sender<StatType>,
) {
    // RDH CHECK: There is always page 0 + minimum page 1 + stop flag
    if let Err(e) = running_rdh_checker.check(&rdh) {
        stats_sender_ch_checker
            .send(StatType::Error(format!("RDH check failed: {}", e)))
            .unwrap();
        let tmp_last_rdh = running_rdh_checker.last_rdh2.unwrap();
        info!("Last RDH: {tmp_last_rdh}");
        info!("Current RDH: {rdh}");
    }
}

pub fn preprocess_payload(payload: &Vec<u8>) -> Result<impl Iterator<Item = &[u8]>, String> {
    // Retrieve padding from payload
    let ff_padding = payload
        .iter()
        .rev()
        .take_while(|&x| *x == 0xFF)
        .collect::<Vec<_>>();

    if ff_padding.len() > 15 {
        return Err(format!("End of payload 0xFF padding is {} bytes, exceeding max of 15 bytes: Skipping current payload",
        ff_padding.len()));
    }

    // Split payload into GBT words sized slices, using chunks_exact to allow more compiler optimizations,
    //   and to cut off any padding less than 10 bytes. More than 9 bytes of padding will be removed in the statement below
    let gbt_word_chunks = if ff_padding.len() > 9 {
        // If the padding is more than 9 bytes, it will be processed as a GBT word, therefor exclude it from the slice
        let last_idx = payload.len(); // Not (len-1) because of how rust slice indexing work
        let last_idx_before_padding = last_idx - ff_padding.len();
        let chunks = payload[..last_idx_before_padding].chunks_exact(10);
        debug_assert!(chunks.remainder().len() == 0);
        chunks
    } else {
        let chunks = payload.as_slice().chunks_exact(10);
        debug_assert!(chunks.remainder().iter().all(|&x| x == 0xFF)); // Asserts that the payload padding is 0xFF
        chunks
    };

    Ok(gbt_word_chunks)
}
