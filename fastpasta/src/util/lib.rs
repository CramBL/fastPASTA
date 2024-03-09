//! Miscellaneous utility functions
use crate::util::*;

/// Start the [stderrlog] instance, and immediately use it to log the configured [DataOutputMode].
pub fn init_error_logger(cfg: &(impl UtilOpt + InputOutputOpt)) {
    stderrlog::new()
        .module("fastpasta")
        .verbosity(cfg.verbosity() as usize)
        .init()
        .expect("Failed to initialize logger");
    match cfg.output_mode() {
        DataOutputMode::Stdout => log::trace!("Data ouput set to stdout"),
        DataOutputMode::File(p) => log::trace!("Data ouput set to file: {:?}", p),
        DataOutputMode::None => {
            log::trace!("Data output set to suppressed")
        }
    }
    log::trace!("Starting fastpasta with args: {:#?}", Cfg::global());
    log::trace!("Checks enabled: {:#?}", Cfg::global().check());
    log::trace!("Views enabled: {:#?}", Cfg::global().view());
}

/// Initializes the Ctrl+C handler to facilitate graceful shutdown on Ctrl+C
///
/// Also handles SIGTERM and SIGHUP if the `termination` feature is enabled
pub fn init_ctrlc_handler(stop_flag: Arc<AtomicBool>) {
    // Handles SIGINT, SIGTERM and SIGHUP (as the `termination` feature is  enabled)
    ctrlc::set_handler({
        let mut stop_sig_count = 0;
        move || {
            log::warn!(
                "Stop Ctrl+C, SIGTERM, or SIGHUP received, stopping gracefully, please wait..."
            );
            stop_flag.store(true, Ordering::SeqCst);
            stop_sig_count += 1;
            if stop_sig_count > 1 {
                log::warn!("Second stop signal received, ungraceful shutdown.");
                std::process::exit(1);
            }
        }
    })
    .expect("Error setting Ctrl-C handler");
}

/// Exits the program with the appropriate exit code
pub fn exit(exit_code: u8, any_errors_flag: &AtomicBool) -> ExitCode {
    if exit_code == 0 {
        log::debug!("Exit successful from data processing");
        if Cfg::global().any_errors_exit_code().is_some() && any_errors_flag.load(Ordering::Relaxed)
        {
            ExitCode::from(Cfg::global().any_errors_exit_code().unwrap())
        } else {
            ExitCode::SUCCESS
        }
    } else {
        ExitCode::from(exit_code)
    }
}
