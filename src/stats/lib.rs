//! Contains the [init_stats_controller] function, which spawns a thread with the [StatsController] running, and returns the thread handle, the channel to send stats to, and the stop flag.
use super::stats_controller::{self, StatsController};
use crate::{util::lib::Config, words};
use std::sync::atomic::AtomicBool;

/// Spawns a thread with the StatsController running, and returns the thread handle, the channel to send stats to, and the stop flag.
pub fn init_stats_controller<C: Config + 'static>(
    config: std::sync::Arc<C>,
) -> (
    std::thread::JoinHandle<()>,
    std::sync::mpsc::Sender<super::stats_controller::StatType>,
    std::sync::Arc<AtomicBool>,
) {
    let mut stats = StatsController::new(config);
    let send_stats_channel = stats.send_channel();
    let thread_stop_flag = stats.end_processing_flag();
    let stats_thread = std::thread::Builder::new()
        .name("stats_thread".to_string())
        .spawn(move || {
            stats.run();
        })
        .expect("Failed to spawn stats thread");
    (stats_thread, send_stats_channel, thread_stop_flag)
}

// Stat collection functions

/// Collects stats specific to ITS from the given [RDH][words::lib::RDH] and sends them to the [StatsController].
pub fn collect_its_stats<T: words::lib::RDH>(
    rdh: &T,
    stats_sender_channel: &std::sync::mpsc::Sender<stats_controller::StatType>,
) {
    let layer = words::its::layer_from_feeid(rdh.fee_id());
    let stave = words::its::stave_number_from_feeid(rdh.fee_id());
    stats_sender_channel
        .send(stats_controller::StatType::LayerStaveSeen { layer, stave })
        .unwrap();
    stats_sender_channel
        .send(stats_controller::StatType::DataFormat(rdh.data_format()))
        .unwrap();
}
