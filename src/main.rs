use fastpasta::{
    input::lib::init_reader,
    stats::lib::{init_stats_controller, StatType},
    util::{
        config::Cfg,
        lib::{ChecksOpt, ViewOpt},
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
    let (stat_controller, stat_send_channel, stop_flag) = init_stats_controller(Cfg::global());

    let exit_code: std::process::ExitCode = match init_reader(Cfg::global()) {
        Ok(readable) => {
            fastpasta::init_processing(Cfg::global(), readable, stat_send_channel, stop_flag)
        }
        Err(e) => {
            stat_send_channel
                .send(StatType::Fatal(e.to_string()))
                .unwrap();
            drop(stat_send_channel);
            std::process::ExitCode::from(1)
        }
    };

    stat_controller.join().expect("Failed to join stats thread");
    exit_code
}
