use fastpasta::{
    input::lib::init_reader,
    stats::lib::{init_stats_controller, StatType},
};

pub fn main() -> std::process::ExitCode {
    let config = fastpasta::get_config();
    fastpasta::init_error_logger(&config);
    log::trace!("Starting fastpasta with args: {config:#?}");
    log::trace!(
        "Checks enabled: {:#?}",
        fastpasta::util::lib::ChecksOpt::check(&config)
    );
    log::trace!(
        "Views enabled: {:#?}",
        fastpasta::util::config::view::ViewOpt::view(&config)
    );

    // Launch statistics thread
    // If max allowed errors is reached, stop the processing from the stats thread
    let (stat_controller, stat_send_channel, stop_flag) = init_stats_controller(config.clone());

    let exit_code: std::process::ExitCode = match init_reader(&config) {
        Ok(readable) => fastpasta::init_processing(config, readable, stat_send_channel, stop_flag),
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
