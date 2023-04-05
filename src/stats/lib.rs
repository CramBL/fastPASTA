//! Contains the [init_stats_controller] function, which spawns a thread with the [StatsController] running, and returns the thread handle, the channel to send stats to, and the stop flag.
use super::stats_controller::StatsController;
use crate::util::lib::Config;
use std::sync::atomic::AtomicBool;

/// Spawns a thread with the StatsController running, and returns the thread handle, the channel to send stats to, and the stop flag.
pub fn init_stats_controller(
    config: &impl Config,
) -> (
    std::thread::JoinHandle<()>,
    std::sync::mpsc::Sender<super::stats_controller::StatType>,
    std::sync::Arc<AtomicBool>,
) {
    let (send_stats_channel, recv_stats_channel): (
        std::sync::mpsc::Sender<super::stats_controller::StatType>,
        std::sync::mpsc::Receiver<super::stats_controller::StatType>,
    ) = std::sync::mpsc::channel();
    let thread_stop_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mut stats = StatsController::new(config, recv_stats_channel, thread_stop_flag.clone());
    let stats_thread = std::thread::Builder::new()
        .name("stats_thread".to_string())
        .spawn(move || {
            stats.run();
        })
        .expect("Failed to spawn stats thread");
    (stats_thread, send_stats_channel, thread_stop_flag)
}
