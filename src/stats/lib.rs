use super::stats_controller::Stats;
use crate::util::config::Opt;
use std::sync::atomic::AtomicBool;

pub fn init_stats_controller(
    config: &Opt,
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
    let mut stats = Stats::new(config, recv_stats_channel, thread_stop_flag.clone());
    let stats_thread = std::thread::Builder::new()
        .name("stats_thread".to_string())
        .spawn(move || {
            stats.run();
        })
        .expect("Failed to spawn stats thread");
    (stats_thread, send_stats_channel, thread_stop_flag)
}
