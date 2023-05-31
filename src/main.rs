use fastpasta::{
    input::lib::init_reader,
    stats::lib::{init_stats_controller, StatType},
    util::{
        config::Cfg,
        lib::{ChecksOpt, UtilOpt, ViewOpt},
    },
};

pub fn main() -> std::process::ExitCode {
    if let Err(e) = fastpasta::init_config() {
        eprintln!("{e}");
        return std::process::ExitCode::from(1);
    };

    fastpasta::init_error_logger(Cfg::global());
    log::trace!("Starting fastpasta with args: {:#?}", Cfg::global());
    log::trace!("Checks enabled: {:#?}", Cfg::global().check());
    log::trace!("Views enabled: {:#?}", Cfg::global().view());

    // Launch statistics thread
    // If max allowed errors is reached, stop the processing from the stats thread
    let (stat_controller, stat_send_channel, stop_flag, any_errors_flag) =
        init_stats_controller(Cfg::global());

    let exit_code: u8 = match init_reader(Cfg::global()) {
        Ok(readable) => {
            match fastpasta::init_processing(Cfg::global(), readable, stat_send_channel, stop_flag)
            {
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

    if exit_code == 0 {
        log::info!("Exit successful from data processing");
        if let Some(custom_exit_code) = Cfg::global().exit_code_any_errors() {
            if any_errors_flag.load(std::sync::atomic::Ordering::Relaxed) {
                std::process::ExitCode::from(custom_exit_code)
            } else {
                std::process::ExitCode::SUCCESS
            }
        } else {
            std::process::ExitCode::SUCCESS
        }
    } else {
        std::process::ExitCode::from(exit_code)
    }
}
