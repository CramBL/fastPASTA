use fastpasta::{
    input::{
        bufreader_wrapper::BufferedReaderWrapper, input_scanner::InputScanner, lib::init_reader,
    },
    stats::{lib::init_stats_controller, stats_controller},
    util::lib::Config,
    words::{
        lib::RdhSubWord,
        rdh::Rdh0,
        rdh_cru::{RdhCRU, V6, V7},
    },
};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub fn main() -> std::process::ExitCode {
    let config = fastpasta::get_config();
    fastpasta::init_error_logger(&*config);
    log::trace!("Starting fastpasta with args: {:#?}", config);
    log::trace!(
        "Checks enabled: {:#?}",
        fastpasta::util::lib::Checks::check(&*config)
    );
    log::trace!(
        "Views enabled: {:#?}",
        fastpasta::util::lib::Views::view(&*config)
    );

    // Launch statistics thread
    // If max allowed errors is reached, stop the processing from the stats thread
    let (stat_controller, stat_send_channel, stop_flag) = init_stats_controller(config.clone());

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

fn init_processing(
    config: Arc<impl Config + 'static>,
    mut reader: Box<dyn BufferedReaderWrapper>,
    stat_send_channel: std::sync::mpsc::Sender<stats_controller::StatType>,
    thread_stopper: Arc<AtomicBool>,
) -> std::process::ExitCode {
    // Determine RDH version
    let rdh0 = Rdh0::load(&mut reader).expect("Failed to read first RDH0");
    let rdh_version = rdh0.header_id;

    // Send RDH version to stats thread
    stat_send_channel
        .send(stats_controller::StatType::RdhVersion(rdh_version))
        .unwrap();

    // Create input scanner from the already read RDH0 (to avoid seeking back and reading it twice, which would also break with stdin piping)
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
            Ok(_) => fastpasta::exit_success(),
            Err(e) => exit_fatal(stat_send_channel, e.to_string(), 2),
        },
        7 => match fastpasta::process::<RdhCRU<V7>>(
            config,
            loader,
            stat_send_channel.clone(),
            thread_stopper,
        ) {
            Ok(_) => fastpasta::exit_success(),
            Err(e) => exit_fatal(stat_send_channel, e.to_string(), 2),
        },
        _ => exit_fatal(
            stat_send_channel,
            format!("Unknown RDH version: {rdh_version}"),
            3,
        ),
    }
}

fn exit_fatal(
    stat_send_channel: std::sync::mpsc::Sender<stats_controller::StatType>,
    error_string: String,
    exit_code: u8,
) -> std::process::ExitCode {
    stat_send_channel
        .send(stats_controller::StatType::Fatal(error_string))
        .unwrap();
    std::process::ExitCode::from(exit_code)
}
