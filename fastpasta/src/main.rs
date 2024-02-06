use alice_protocol_reader::init_reader;
use fastpasta::config::init_config;
use fastpasta::config::prelude::*;
use fastpasta::config::Cfg;
use fastpasta::controller::init_controller;
use fastpasta::stats::StatType;

pub fn main() -> std::process::ExitCode {
    if let Err(e) = init_config() {
        eprintln!("{e}");
        return std::process::ExitCode::from(1);
    };

    fastpasta::util::lib::init_error_logger(Cfg::global());

    if Cfg::global().generate_custom_checks_toml_enabled() {
        log::warn!("'custom_checks.toml' file generated in current directory. Use it to customize checks. Exiting...");
        return std::process::ExitCode::from(0);
    }

    if let Some(shell) = Cfg::global().generate_completions {
        Cfg::generate_completion_script(shell);
        log::warn!("Completions generated for {shell:?}. Exiting...");
        return std::process::ExitCode::from(0);
    }

    // Launch controller thread
    // If max allowed errors is reached, the controller thread signals every other thread to stop
    let (controller, stat_send_chan, stop_flag, any_errors_flag) = init_controller(Cfg::global());

    // Handles SIGINT, SIGTERM and SIGHUP (as the `termination` feature is  enabled)
    fastpasta::util::lib::init_ctrlc_handler(stop_flag.clone());

    let exit_code: u8 = match init_reader(Cfg::global().input_file()) {
        Ok(readable) => {
            match fastpasta::init_processing(Cfg::global(), readable, stat_send_chan, stop_flag) {
                Ok(_) => 0,
                Err(e) => {
                    log::error!("Init processing failed: {e}");
                    1
                }
            }
        }
        Err(e) => {
            stat_send_chan
                .send(StatType::Fatal(e.to_string().into()))
                .unwrap();
            drop(stat_send_chan);
            1
        }
    };

    controller.join().expect("Failed to join stats thread");

    fastpasta::util::lib::exit(exit_code, &any_errors_flag)
}
