use std::vec;

use fastpasta::data_words::rdh::{Rdh0, RdhCRUv6, RdhCRUv7};
use fastpasta::util::config::Opt;
use fastpasta::util::file_scanner::{FileScanner, ScanCDP};
use fastpasta::util::stats;
use fastpasta::util::writer::{BufferedWriter, Writer};
use fastpasta::{buf_reader_with_capacity, file_open_read_only, GbtWord, RDH};
use structopt::StructOpt;

pub enum SeekError {
    EOF,
}

pub fn main() -> std::io::Result<()> {
    let opt: Opt = StructOpt::from_args();
    println!("{:#?}", opt);

    // Determine RDH version
    let file = file_open_read_only(opt.file())?;
    let mut reader = buf_reader_with_capacity(file, 256);
    let rdh0 = Rdh0::load(&mut reader)?;
    // Choose the rest of the execution based on the RDH version
    // Necessary to prevent heap allocation and allow static dispatch as the type cannot be known at compile time
    match rdh0.header_id {
        6 => process_rdh_v6(opt).unwrap(),
        7 => process_rdh_v7(opt).unwrap(),
        _ => panic!("Unknown RDH version: {}", rdh0.header_id),
    }

    // 1. Create reader: FileScanner (contains FilePosTracker and borrows Stats)
    //      - Open file in read only mode
    //      - Wrap in BufReader
    //      - Track file position (FilePosTracker)
    //      - reads data through struct interface + buffer
    //      - collects stats (Stats)
    // 2. Read into a reasonably sized buffer (TODO)
    // 3. Pass buffer to checker and read another chunk (TODO)
    // 4. Checker verifies received buffered chunk (big checks -> multi-threading)
    //                Not valid -> Print error and abort
    //                Valid     -> Pass chunk to writer
    // 5. Writer writes chunk to file OR stdout

    Ok(())
}

// 1. Setup reading (file or stdin) // TODO: stdin support
// 2. Do checks on read data
// 3. Write data out (file or stdout)
pub fn process_rdh_v7(config: Opt) -> std::io::Result<()> {
    type V7 = RdhCRUv7;
    // Setup reader, checker, writer, stats
    let mut running_rdh_checker = fastpasta::validators::rdh::RdhCruv7RunningChecker::new();
    let mut stats = stats::Stats::new();
    // Automatically extracts link to filter if one is supplied
    let mut file_scanner = FileScanner::default(&config, &mut stats);
    let mut writer = BufferedWriter::<V7>::new(&config, 1024 * 1024); // 1MB buffer

    loop {
        // 1. Reading
        // let (rdh_chunk, payload_chunk) =
        //     get_chunk::<V7>(&mut file_scanner, 10).expect("Error reading CDP chunks");

        let (rdh_chunk, payload_chunk) = match get_chunk::<V7>(&mut file_scanner, 10) {
            Ok(cdp) => cdp,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    break;
                } else {
                    panic!("Error reading CDP chunks: {}", e);
                }
            }
        };

        // 2. Checking
        for rdh in rdh_chunk.as_slice() {
            if config.sanity_checks() {
                sanity_validation(&rdh);
            }
            // RDH CHECK: There is always page 0 + minimum page 1 + stop flag
            match running_rdh_checker.check(&rdh) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("RDH check failed: {}", e);
                    RdhCRUv7::print_header_text();
                    eprintln!("Last RDH:");
                    running_rdh_checker.last_rdh2.unwrap().print();
                    eprintln!("Current RDH:");
                    rdh.print();
                }
            }
        }

        for payload in payload_chunk.as_slice() {
            if config.sanity_checks() {
                // TODO: Sanity check for payload
            }
        }

        // 3. Writing
        writer.push_cdps_raw((rdh_chunk, payload_chunk));
    }

    stats.print();
    Ok(())
}

pub fn process_rdh_v6(config: Opt) -> std::io::Result<()> {
    todo!("RDH v6 not implemented yet");
    let mut stats = stats::Stats::new();
    // Automatically extracts link to filter if one is supplied
    let mut file_scanner = FileScanner::default(&config, &mut stats);

    let (rdh_chunk, payload_chunk) =
        get_chunk::<RdhCRUv6>(&mut file_scanner, 10).expect("Error reading CDP chunks");

    for rdh in rdh_chunk {
        if config.sanity_checks() {
            todo!("Sanity check for RDH v6")
        }
    }

    Ok(())
}

pub fn sanity_validation(rdh: &RdhCRUv7) {
    let rdh_validator = fastpasta::validators::rdh::RDH_CRU_V7_VALIDATOR;
    match rdh_validator.sanity_check(&rdh) {
        Ok(_) => (),
        Err(e) => {
            println!("Sanity check failed: {}", e);
        }
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
                eprintln!("EOF reached! ");
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
