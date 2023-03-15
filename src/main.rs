#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_code)]
use fastpasta::input::{
    bufreader_wrapper::BufferedReaderWrapper, input_scanner::InputScanner, lib::init_reader,
};
use fastpasta::stats::{lib::init_stats_controller, stats_controller};
use fastpasta::util::config::Opt;
use fastpasta::words::{
    lib::RdhSubWord,
    rdh::Rdh0,
    rdh_cru::{RdhCRU, V6, V7},
};
use fastpasta::{data_write, validators};
use log::trace;
use std::io;
use std::sync::atomic::AtomicBool;
use std::{sync::Arc, thread::JoinHandle};
use structopt::StructOpt;

pub fn main() {
    let config = get_config();
    init_error_logger(&config);
    trace!("Starting fastpasta with args: {:#?}", config);

    // Launch statistics thread
    // If max allowed errors is reached, stop the processing from the stats thread
    let (stat_controller, stat_send_channel, stop_flag) = init_stats_controller(&config);

    let mut readable = init_reader(&config);

    // Determine RDH version
    let rdh0 = Rdh0::load(&mut readable).expect("Failed to read first RDH0");
    let rdh_version = rdh0.header_id;
    stat_send_channel
        .send(stats_controller::StatType::RdhVersion(rdh_version))
        .unwrap();
    let loader =
        InputScanner::new_from_rdh0(config.clone(), readable, stat_send_channel.clone(), rdh0);

    // Choose the rest of the execution based on the RDH version
    // Necessary to prevent heap allocation and allow static dispatch as the type cannot be known at compile time
    match rdh_version {
        6 => {
            log::warn!("RDH version 6 detected, using RDHv7 processing for now anyways... No guarantees it will work!");
            process::<RdhCRU<V6>>(config, loader, stat_send_channel, stop_flag).unwrap()
        }
        7 => process::<RdhCRU<V7>>(config, loader, stat_send_channel, stop_flag).unwrap(),
        _ => panic!("Unknown RDH version: {rdh_version}"),
    }
    stat_controller.join().expect("Failed to join stats thread");
}

fn init_error_logger(cfg: &Opt) {
    stderrlog::new()
        .module(module_path!())
        .verbosity(cfg.verbosity() as usize)
        .init()
        .expect("Failed to initialize logger");
    match cfg.output_mode() {
        fastpasta::util::config::DataOutputMode::Stdout => trace!("Data ouput set to stdout"),
        fastpasta::util::config::DataOutputMode::File => trace!("Data ouput set to file"),
        fastpasta::util::config::DataOutputMode::None => trace!("Data ouput set to suppressed"),
    }
}

fn get_config() -> Arc<Opt> {
    let mut cfg = Opt::from_args();

    if let Err(e) = cfg.arg_validate() {
        eprintln!("{e}");
        std::process::exit(1);
    }

    cfg.sort_link_args();

    Arc::new(cfg)
}
// 1. Setup reading (file or stdin)
// 2. Do checks on read data
// 3. Write data out (file or stdout)
pub fn process<T: fastpasta::words::lib::RDH + 'static>(
    config: Arc<Opt>,
    loader: InputScanner<impl BufferedReaderWrapper + ?Sized + std::marker::Send + 'static>,
    send_stats_ch: std::sync::mpsc::Sender<stats_controller::StatType>,
    thread_stopper: Arc<AtomicBool>,
) -> io::Result<()> {
    // 1. Read data from file
    let (reader_handle, reader_rcv_channel) =
        fastpasta::input::lib::spawn_reader(thread_stopper.clone(), loader);

    // 2. Do checks on a received chunk of data
    let (validator_handle, checker_rcv_channel) = validators::lib::spawn_validator::<T>(
        config.clone(),
        thread_stopper.clone(),
        send_stats_ch,
        reader_rcv_channel,
    );

    // 3. Write data out
    let writer_handle: Option<JoinHandle<()>> = match config.output_mode() {
        fastpasta::util::config::DataOutputMode::None => None,
        _ => Some(data_write::lib::spawn_writer(
            config.clone(),
            thread_stopper,
            checker_rcv_channel.expect("Checker receiver channel not initialized"),
        )),
    };

    reader_handle.join().expect("Error joining reader thread");
    if let Err(e) = validator_handle.join() {
        log::error!("Validator thread terminated early: {:#?}\n", e);
    }
    if let Some(writer) = writer_handle {
        writer.join().expect("Could not join writer thread");
    }
    Ok(())
}
