#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_code)]
use fastpasta::util::bufreader_wrapper::BufferedReaderWrapper;
use fastpasta::util::config::Opt;
use fastpasta::util::file_pos_tracker::FilePosTracker;
use fastpasta::util::file_scanner::FileScanner;
use fastpasta::util::process_v7;
use fastpasta::util::stats::{self};
use fastpasta::words::rdh::{Rdh0, RdhCRUv6};
use fastpasta::{init_stats_controller, GbtWord};
use log::trace;
use std::io;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
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
    init_error_logger(&opt);
    trace!("Starting fastpasta with args: {:#?}", opt);
    if let Err(e) = opt.arg_validate() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
    match opt.output_mode() {
        fastpasta::util::config::DataOutputMode::Stdout => trace!("Data ouput set to stdout"),
        fastpasta::util::config::DataOutputMode::File => trace!("Data ouput set to file"),
        fastpasta::util::config::DataOutputMode::None => trace!("Data ouput set to suppressed"),
    }

    let config: Arc<Opt> = Arc::new(opt);

    // Launch statistics thread
    // If max allowed errors is reached, stop the processing from the stats thread
    let (stat_controller, stat_send_channel, stop_flag) = init_stats_controller(&config.clone());

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

// 1. Setup reading (file or stdin) // TODO: stdin support
// 2. Do checks on read data
// 3. Write data out (file or stdout)
pub fn process_rdh_v7(
    config: Arc<Opt>,
    loader: FileScanner<impl BufferedReaderWrapper + ?Sized + std::marker::Send + 'static>,
    send_stats_ch: std::sync::mpsc::Sender<stats::StatType>,
    thread_stopper: Arc<AtomicBool>,
) -> io::Result<()> {
    // 1. Read data from file
    let (reader_handle, reader_rcv_channel) =
        process_v7::input::spawn_reader(thread_stopper.clone(), loader);

    // 2. Do checks on a received chunk of data
    let (validator_handle, checker_rcv_channel) = process_v7::validate::spawn_checker(
        config.clone(),
        thread_stopper.clone(),
        send_stats_ch.clone(),
        reader_rcv_channel,
    );

    // 3. Write data out
    let writer_handle = process_v7::output::spawn_writer(
        config.clone(),
        thread_stopper.clone(),
        send_stats_ch,
        checker_rcv_channel,
    );

    reader_handle.join().expect("Error joining reader thread");
    validator_handle
        .join()
        .expect("Error joining checker thread");
    writer_handle.join().expect("Error joining writer thread");
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
        fastpasta::get_chunk::<RdhCRUv6>(&mut file_scanner, 10).expect("Error reading CDP chunks");

    for _rdh in rdh_chunk {
        if config.sanity_checks() {
            todo!("Sanity check for RDH v6")
        }
    }

    Ok(())
}
