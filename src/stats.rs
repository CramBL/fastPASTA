//! All stat collecting functionality, and controller that can stop the program based on the collected stats.
pub mod its_stats;
pub mod lib;
pub mod rdh_stats;
mod report;
pub mod stats_controller;

/// Displays an error message if the config doesn't have the mute error flag set.
pub fn display_error(error: &str) {
    use crate::util::{config::CONFIG, lib::UtilOpt};
    let is_muting_errors = CONFIG.get().unwrap().mute_errors();
    if !is_muting_errors {
        log::error!("{error}");
    }
}
