#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_code)]
use crossbeam_channel::{bounded, Receiver, RecvError, Sender};
use fastpasta::util::config::Opt;
use fastpasta::util::file_pos_tracker::FilePosTracker;
use fastpasta::util::file_scanner::{FileScanner, ScanCDP};
use fastpasta::util::stats;
use fastpasta::util::writer::{BufferedWriter, Writer};
use fastpasta::validators::cdp_running::CdpRunningValidator;
use fastpasta::validators::rdh::RdhCruv7RunningChecker;
use fastpasta::words::rdh::{Rdh0, RdhCRUv6, RdhCRUv7};
use fastpasta::{
    buf_reader_with_capacity, file_open_read_only, setup_buffered_reading, GbtWord, RDH,
};
use log::{debug, error, info};
use std::sync::Arc;
use std::{thread, vec};
use structopt::StructOpt;

pub fn main() -> std::io::Result<()> {
    let opt: Opt = StructOpt::from_args();

    stderrlog::new()
        .module(module_path!())
        .verbosity(opt.verbosity() as usize)
        .init()
        .unwrap();

    debug!("{:#?}", opt);

    let config = opt;
    // let config: Opt = <Opt as structopt::StructOpt>::from_iter(&[
    //     "fastpasta",
    //     "-s",
    //     "-f",
    //     "0",
    //     "../fastpasta_test_files/data_ols_ul.raw",
    //     "-o test_filter_link.raw",
    // ]);

    let config: Arc<Opt> = Arc::new(config);

    // Determine RDH version
    let file = file_open_read_only(config.file())?;
    let mut reader = buf_reader_with_capacity(file, 256);
    let rdh0 = Rdh0::load(&mut reader)?;
    // Choose the rest of the execution based on the RDH version
    // Necessary to prevent heap allocation and allow static dispatch as the type cannot be known at compile time
    match rdh0.header_id {
        6 => process_rdh_v6(config).unwrap(),
        7 => process_rdh_v7(config).unwrap(),
        _ => panic!("Unknown RDH version: {}", rdh0.header_id),
    }

    // 1. Create reader: FileScanner (contains FilePosTracker and borrows Stats)
    //      - Open file in read only mode
    //      - Wrap in BufReader
    //      - Track file position (FilePosTracker)
    //      - reads data through struct interface + buffer
    //      - collects stats (Stats)
    // 2. Read into a reasonably sized buffer
    // 3. Pass buffer to checker and read another chunk
    // 4. Checker verifies received buffered chunk (big checks -> multi-threading)
    //                Not valid -> Print error and abort
    //                Valid     -> Pass chunk to writer
    // 5. Writer writes chunk to file OR stdout

    Ok(())
}

// 1. Setup reading (file or stdin) // TODO: stdin support
// 2. Do checks on read data
// 3. Write data out (file or stdout)
pub fn process_rdh_v7(config: Arc<Opt>) -> std::io::Result<()> {
    // Types specific for RDH v7
    type V7 = RdhCRUv7;
    type Cdp = (Vec<V7>, Vec<Vec<u8>>);
    // Create producer-consumer channel for the reader to the checker
    let (sender_reader, receiver_checker): (Sender<Cdp>, Receiver<Cdp>) = bounded(100);
    // Create producer-consumer channel for the checker to the writer
    let (sender_checker, receiver_writer): (Sender<Cdp>, Receiver<Cdp>) = bounded(100);

    // Setup reader, checker, writer, stats
    let mut running_rdh_checker = RdhCruv7RunningChecker::new();
    let mut stats = stats::Stats::new();
    let mut writer = BufferedWriter::<V7>::new(&config, 1024 * 1024); // 1MB buffer

    // 1. Read data from file
    let cfg = Arc::clone(&config);
    let reader_thread = thread::spawn(move || {
        let reader = setup_buffered_reading(&cfg);
        // Automatically extracts link to filter if one is supplied
        let mut file_scanner = FileScanner::new(&cfg, reader, FilePosTracker::new(), &mut stats);

        loop {
            let (rdh_chunk, payload_chunk) = match get_chunk::<V7>(&mut file_scanner, 100) {
                Ok(cdp) => cdp,
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::UnexpectedEof {
                        break;
                    } else {
                        panic!("Error reading CDP chunks: {}", e);
                    }
                }
            };
            // Send a chunk to the checker
            sender_reader.send((rdh_chunk, payload_chunk)).unwrap();
        }
        stats.print();
    });

    // 2. Do checks on a received chunk of data
    let cfg = config.clone();
    let checker_thread = thread::spawn(move || {
        let mut cdp_payload_running_validator = CdpRunningValidator::new();

        loop {
            // Receive chunk from reader
            let (rdh_chunk, payload_chunk) = match receiver_checker.recv() {
                Ok(cdp) => cdp,
                Err(e) => {
                    debug_assert_eq!(e, RecvError);
                    break;
                }
            };

            // Do checks one each pair of RDH and payload in the chunk
            for (rdh, payload) in rdh_chunk.iter().zip(payload_chunk.iter()) {
                info!("{rdh}");
                // Check RDH
                if cfg.sanity_checks() {
                    sanity_validation(&rdh);
                }
                do_rdh_v7_running_checks(&rdh, &mut running_rdh_checker);

                // Check padding:
                //  - Flavor 0 will have no padding - other than the usual 6 bytes with 0x0
                //  - Flavor 1 will have padding if last word does not fill all 16 bytes
                // Asserts that the payload is padded to 16 bytes at the end (Fails for data_ols_ul.raw as it is old from when the padding logic was bugged)
                if let Ok(gbt_word_chunks) = preprocess_payload(payload) {
                    gbt_word_chunks.for_each(|gbt_word| {
                        if let Err(e) = cdp_payload_running_validator.check(rdh, gbt_word) {
                            error!(
                                "Payload check failed for: {:?} - With error:{}",
                                gbt_word, e
                            );
                        }
                    });
                } else {
                    cdp_payload_running_validator.reset_fsm();
                    continue;
                }
            }
            // Checks are done, send chunk to writer
            sender_checker.send((rdh_chunk, payload_chunk)).unwrap();
        }
    });

    // 3. Write data out
    let writer_thread = thread::spawn(move || loop {
        // Receive chunk from checker
        let (rdh_chunk, payload_chunk) = match receiver_writer.recv() {
            Ok(cdp) => cdp,
            Err(e) => {
                debug_assert_eq!(e, RecvError);
                break;
            }
        };
        // Push data onto the writer's buffer, which will flush it when the buffer is full or when the writer is dropped
        writer.push_cdps_raw((rdh_chunk, payload_chunk));
    });

    reader_thread.join().unwrap();
    checker_thread.join().unwrap();
    writer_thread.join().unwrap();

    Ok(())
}

pub fn process_rdh_v6(config: Arc<Opt>) -> std::io::Result<()> {
    todo!("RDH v6 not implemented yet");
    let mut stats = stats::Stats::new();
    // Automatically extracts link to filter if one is supplied
    let mut file_scanner = FileScanner::default(&config, &mut stats);

    let (rdh_chunk, _payload_chunk) =
        get_chunk::<RdhCRUv6>(&mut file_scanner, 10).expect("Error reading CDP chunks");

    for _rdh in rdh_chunk {
        if config.sanity_checks() {
            todo!("Sanity check for RDH v6")
        }
    }

    Ok(())
}

pub fn sanity_validation(rdh: &RdhCRUv7) {
    let rdh_validator = fastpasta::validators::rdh::RDH_CRU_V7_VALIDATOR;
    if let Err(e) = rdh_validator.sanity_check(&rdh) {
        error!("Sanity check failed: {}", e);
    }
}

pub fn setup_file_ops<'a>(opt: &'a Opt, stats: &'a mut stats::Stats) -> FileScanner<'a> {
    let file_scanner = FileScanner::default(&opt, stats);
    file_scanner
}

pub fn get_chunk<T: RDH>(
    file_scanner: &mut FileScanner,
    chunk_size_cdps: usize,
) -> Result<(Vec<T>, Vec<Vec<u8>>), std::io::Error> {
    let mut rdhs: Vec<T> = vec![];
    let mut payloads: Vec<Vec<u8>> = vec![];

    for _ in 0..chunk_size_cdps {
        let (rdh, payload) = match file_scanner.load_cdp() {
            Ok(cdp) => cdp,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                info!("EOF reached! ");
                break;
            }
            Err(e) => return Err(e),
        };
        rdhs.push(rdh);
        payloads.push(payload);
    }

    if rdhs.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "No CDPs found",
        ));
    }

    Ok((rdhs, payloads))
}

pub fn do_rdh_v7_running_checks(rdh: &RdhCRUv7, running_rdh_checker: &mut RdhCruv7RunningChecker) {
    // RDH CHECK: There is always page 0 + minimum page 1 + stop flag
    if let Err(e) = running_rdh_checker.check(&rdh) {
        error!("RDH check failed: {}", e);
        let tmp_last_rdh = running_rdh_checker.last_rdh2.unwrap();
        info!("Last RDH: {tmp_last_rdh}");
        info!("Current RDH: {rdh}");
    }
}

pub fn preprocess_payload(payload: &Vec<u8>) -> Result<impl Iterator<Item = &[u8]>, ()> {
    // Retrieve padding from payload
    let ff_padding = payload
        .iter()
        .rev()
        .take_while(|&x| *x == 0xFF)
        .collect::<Vec<_>>();

    if ff_padding.len() > 15 {
        error!("End of payload 0xFF padding is {} bytes, exceeding max of 15 bytes: Skipping current payload",
        ff_padding.len());
        return Err(());
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
