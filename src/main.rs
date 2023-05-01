use fastpasta::{
    input::lib::init_reader,
    stats::lib::{init_stats_controller, StatType},
    util::{
        self,
        config::Cfg,
        lib::{Checks, InputOutput, Views},
    },
};

pub fn main() -> std::process::ExitCode {
    let config = util::config::Cfg::from_cli_args();
    util::config::CONFIG.set(config).unwrap();
    fastpasta::init_error_logger();
    log::trace!("Starting fastpasta with args: {:#?}", Cfg::global());
    log::trace!("Checks enabled: {:#?}", Cfg::global().check());
    log::trace!("Views enabled: {:#?}", Cfg::global().view());

    // Launch statistics thread
    // If max allowed errors is reached, stop the processing from the stats thread
    let (stat_controller, stat_send_channel, stop_flag) =
        init_stats_controller(Some(Cfg::global()));

    let exit_code: std::process::ExitCode = match init_reader(Cfg::global().input_file()) {
        Ok(readable) => fastpasta::init_processing(readable, stat_send_channel, stop_flag),
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
