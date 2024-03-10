//! Contains the [run] function that is the entry point for fastPASTA
use self::lib::{init_ctrlc_handler, init_error_logger};
use crate::{config::init_config, controller::init_controller, init_processing, util::*};
use alice_protocol_reader::init_reader;

/// Entry point for fastPASTA
pub fn run() -> ExitCode {
    human_panic::setup_panic!();

    if let Err(e) = init_config() {
        eprintln!("{e}");
        return ExitCode::from(1);
    };

    init_error_logger(Cfg::global());

    if Cfg::global().generate_custom_checks_toml_enabled() {
        log::warn!("'custom_checks.toml' file generated in current directory. Use it to customize checks. Exiting...");
        return ExitCode::from(0);
    }

    if let Some(shell) = Cfg::global().generate_completions {
        Cfg::generate_completion_script(shell);
        log::warn!("Completions generated for {shell:?}. Exiting...");
        return ExitCode::from(0);
    }

    // Launch controller thread
    // If max allowed errors is reached, the controller thread signals every other thread to stop
    let (controller, stat_send_chan, stop_flag, any_errors_flag) = init_controller(Cfg::global());

    // Handles SIGINT, SIGTERM and SIGHUP (as the `termination` feature is  enabled)
    init_ctrlc_handler(stop_flag.clone());

    let exit_code: u8 = match init_reader(Cfg::global().input_file()) {
        Ok(readable) => match init_processing(Cfg::global(), readable, stat_send_chan, stop_flag) {
            Ok(_) => 0,
            Err(e) => {
                log::error!("Init processing failed: {e}");
                1
            }
        },
        Err(e) => {
            stat_send_chan
                .send(StatType::Fatal(e.to_string().into()))
                .unwrap();
            drop(stat_send_chan);
            1
        }
    };

    controller.join().expect("Failed to join stats thread");

    lib::exit(exit_code, &any_errors_flag)
}
