use fastpasta_processing::config::init_config;
use fastpasta_processing::config::prelude::*;
use fastpasta_processing::config::Cfg;
use fastpasta_processing::input::lib::init_reader;
use fastpasta_processing::stats::init_stats_controller;
use fastpasta_processing::stats::StatType;

pub fn main() -> std::process::ExitCode {
    if let Err(e) = init_config() {
        eprintln!("{e}");
        return std::process::ExitCode::from(1);
    };

    fastpasta_processing::util::lib::init_error_logger(Cfg::global());

    if Cfg::global().generate_custom_checks_toml_enabled() {
        log::info!("'custom_checks.toml' file generated in current directory. Use it to customize checks. Exiting...");
        return std::process::ExitCode::from(0);
    }

    // Launch statistics thread
    // If max allowed errors is reached, stop the processing from the stats thread
    let (stat_controller, stat_send_channel, stop_flag, any_errors_flag) =
        init_stats_controller(Cfg::global());

    // Handles SIGINT, SIGTERM and SIGHUP (as the `termination` feature is  enabled)
    fastpasta_processing::util::lib::init_ctrlc_handler(stop_flag.clone());

    let exit_code: u8 = match init_reader(Cfg::global().input_file()) {
        Ok(readable) => {
            match fastpasta_processing::init_processing(
                Cfg::global(),
                readable,
                stat_send_channel,
                stop_flag,
            ) {
                Ok(_) => 0,
                Err(e) => {
                    log::error!("Init processing failed: {e}");
                    1
                }
            }
        }
        Err(e) => {
            stat_send_channel
                .send(StatType::Fatal(e.to_string()))
                .unwrap();
            drop(stat_send_channel);
            1
        }
    };

    stat_controller.join().expect("Failed to join stats thread");

    fastpasta_processing::util::lib::exit(exit_code, any_errors_flag)
}
