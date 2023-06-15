//! All stat collecting functionality, and controller that can stop the program based on the collected stats.
//!
//! Contains the [init_stats_controller] function, which spawns a thread with the [StatsController](stats_controller::StatsController) running, and returns the thread handle, the channel to send stats to, and the stop flag.

pub mod its_stats;
pub mod lib;
pub mod rdh_stats;
mod stat_format_utils;
mod stat_summerize_utils;
pub mod stats_controller;
mod stats_report;

/// Spawns a thread with the StatsController running, and returns the thread handle, the channel to send stats to, and the stop flag.
pub fn init_stats_controller<C: crate::util::lib::Config + 'static>(
    config: &'static C,
) -> (
    std::thread::JoinHandle<()>,
    flume::Sender<lib::StatType>,
    std::sync::Arc<std::sync::atomic::AtomicBool>,
    std::sync::Arc<std::sync::atomic::AtomicBool>,
) {
    log::trace!("Initializing stats controller");
    let mut stats = stats_controller::StatsController::new(config);
    let send_stats_channel = stats.send_channel();
    let thread_stop_flag = stats.end_processing_flag();
    let any_errors_flag = stats.any_errors_flag();

    let stats_thread = std::thread::Builder::new()
        .name("stats_thread".to_string())
        .spawn(move || {
            stats.run();
        })
        .expect("Failed to spawn stats thread");
    (
        stats_thread,
        send_stats_channel,
        thread_stop_flag,
        any_errors_flag,
    )
}

/// Displays an error message if the config doesn't have the mute error flag set.
pub fn display_error(error: &str) {
    use crate::util::{config::CONFIG, lib::UtilOpt};
    let is_muting_errors = CONFIG.get().unwrap().mute_errors();
    if !is_muting_errors {
        log::error!("{error}");
    }
}
