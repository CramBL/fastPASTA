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
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use structopt::StructOpt;

pub fn main() -> std::process::ExitCode {
    let config = get_config();
    init_error_logger(&config);
    log::trace!("Starting fastpasta with args: {:#?}", config);

    // Launch statistics thread
    // If max allowed errors is reached, stop the processing from the stats thread
    let (stat_controller, stat_send_channel, stop_flag) = init_stats_controller(&config);

    let exit_code: std::process::ExitCode = match init_reader(&config) {
        Ok(readable) => init_processing(config, readable, stat_send_channel, stop_flag),
        Err(e) => {
            stat_send_channel
                .send(stats_controller::StatType::Fatal(e.to_string()))
                .unwrap();
            drop(stat_send_channel);
            std::process::ExitCode::from(1)
        }
    };

    stat_controller.join().expect("Failed to join stats thread");
    exit_code
}

fn init_error_logger(cfg: &Opt) {
    stderrlog::new()
        .module(module_path!())
        .verbosity(cfg.verbosity() as usize)
        .init()
        .expect("Failed to initialize logger");
    match cfg.output_mode() {
        fastpasta::util::config::DataOutputMode::Stdout => log::trace!("Data ouput set to stdout"),
        fastpasta::util::config::DataOutputMode::File => log::trace!("Data ouput set to file"),
        fastpasta::util::config::DataOutputMode::None => {
            log::trace!("Data ouput set to suppressed")
        }
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

fn init_processing(
    config: Arc<Opt>,
    mut reader: Box<dyn BufferedReaderWrapper>,
    stat_send_channel: std::sync::mpsc::Sender<stats_controller::StatType>,
    thread_stopper: Arc<AtomicBool>,
) -> std::process::ExitCode {
    // Determine RDH version
    let rdh0 = Rdh0::load(&mut reader).expect("Failed to read first RDH0");
    let rdh_version = rdh0.header_id;
    stat_send_channel
        .send(stats_controller::StatType::RdhVersion(rdh_version))
        .unwrap();
    let loader =
        InputScanner::new_from_rdh0(config.clone(), reader, stat_send_channel.clone(), rdh0);

    // Choose the rest of the execution based on the RDH version
    // Necessary to prevent heap allocation and allow static dispatch as the type cannot be known at compile time
    match rdh_version {
        6 => match fastpasta::process::<RdhCRU<V6>>(
            config,
            loader,
            stat_send_channel.clone(),
            thread_stopper,
        ) {
            Ok(_) => std::process::ExitCode::SUCCESS,
            Err(e) => {
                stat_send_channel
                    .send(stats_controller::StatType::Fatal(e.to_string()))
                    .unwrap();
                std::process::ExitCode::from(2)
            }
        },
        7 => match fastpasta::process::<RdhCRU<V7>>(
            config,
            loader,
            stat_send_channel.clone(),
            thread_stopper,
        ) {
            Ok(_) => std::process::ExitCode::SUCCESS,
            Err(e) => {
                stat_send_channel
                    .send(stats_controller::StatType::Fatal(e.to_string()))
                    .unwrap();
                std::process::ExitCode::from(2)
            }
        },
        _ => {
            stat_send_channel
                .send(stats_controller::StatType::Fatal(format!(
                    "Unknown RDH version: {rdh_version}",
                )))
                .unwrap();
            std::process::ExitCode::from(3)
        }
    }
}
