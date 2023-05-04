//! Contains the [StatsController] that collects stats and reports errors.
//! It also controls the stop flag, which can be used to stop the program if a fatal error occurs, or if the config contains a max number of errors to tolerate.
//! Finally when the event loop breaks (at the end of execution), it will print a summary of the stats collected, using the Report struct.

use super::lib::{StatType, SystemId};
use crate::{
    stats::report::{Report, StatSummary},
    util::lib::Config,
};
use log::error;
use std::sync::{
    atomic::{AtomicBool, AtomicU32},
    Arc,
};

/// The StatsController receives stats and builds a summary report that is printed at the end of execution.
pub struct StatsController<C: Config> {
    /// Total RDHs seen.
    pub rdhs_seen: u64,
    /// Total RDHs filtered.
    pub rdhs_filtered: u64,
    /// Total payload size.
    pub payload_size: u64,
    /// Links observed.
    pub links_observed: Vec<u8>,
    /// Time from [StatsController] is instantiated, to all data processing threads disconnected their [StatType] producer channel.
    pub processing_time: std::time::Instant,
    config: std::sync::Arc<C>,
    total_errors: AtomicU32,
    non_atomic_total_errors: u64,
    max_tolerate_errors: u32,
    // The channel where stats are received from other threads.
    recv_stats_channel: std::sync::mpsc::Receiver<StatType>,
    // The channel stats are sent through, stored so that a clone of the channel can be returned easily
    // Has to be an option so that it can be set to None when the event loop starts.
    // Once run is called no producers that don't already have a channel to send stats through, will be able to get one.
    // This is because the event loop breaks when all sender channels are dropped, and if the StatsController keeps a reference to the channel, it will cause a deadlock.
    send_stats_channel: Option<std::sync::mpsc::Sender<StatType>>,
    end_processing_flag: Arc<AtomicBool>,
    rdh_version: u8,
    data_formats_observed: Vec<u8>,
    hbfs_seen: u32,
    fatal_error: Option<String>,
    layers_staves_seen: Vec<(u8, u8)>,
    run_trigger_type: (u32, String),
    system_id_observed: Option<SystemId>,
}
impl<C: Config> StatsController<C> {
    /// Creates a new StatsController from a [Config], a [std::sync::mpsc::Receiver] for [StatType], and a [std::sync::Arc] of an [AtomicBool] that is used to signal to other threads to exit if a fatal error occurs.
    pub fn new(global_config: std::sync::Arc<C>) -> Self {
        let (send_stats_channel, recv_stats_channel): (
            std::sync::mpsc::Sender<StatType>,
            std::sync::mpsc::Receiver<StatType>,
        ) = std::sync::mpsc::channel();
        StatsController {
            rdhs_seen: 0,
            rdhs_filtered: 0,
            payload_size: 0,
            config: global_config.clone(),
            links_observed: Vec::new(),
            processing_time: std::time::Instant::now(),
            total_errors: AtomicU32::new(0),
            non_atomic_total_errors: 0,
            max_tolerate_errors: global_config.max_tolerate_errors(),
            recv_stats_channel,
            send_stats_channel: Some(send_stats_channel),
            end_processing_flag: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            rdh_version: 0,
            data_formats_observed: Vec::new(),
            hbfs_seen: 0,
            fatal_error: None,
            layers_staves_seen: Vec::new(),
            run_trigger_type: (0, String::from("")),
            system_id_observed: None,
        }
    }

    /// Returns a clone of the channel that is used to send stats to the StatsController.
    pub fn send_channel(&self) -> std::sync::mpsc::Sender<StatType> {
        if self.send_stats_channel.is_none() {
            log::error!("StatsController send channel is none, most likely it is already running and does not accept new producers");
            panic!("StatsController send channel is none, most likely it is already running and does not accept new producers");
        }
        self.send_stats_channel.as_ref().unwrap().clone()
    }

    /// Returns a cloned reference to the end processing flag.
    pub fn end_processing_flag(&self) -> Arc<AtomicBool> {
        self.end_processing_flag.clone()
    }

    /// Starts the event loop for the StatsController
    /// This function will block until the channel is closed
    pub fn run(&mut self) {
        // Set the send stats channel to none so that no new producers can be added, and so the loop breaks when all producers have dropped their channel.
        self.send_stats_channel = None;

        // While loop breaks when an error is received from the channel, which means the channel is disconnected
        while let Ok(stats_update) = self.recv_stats_channel.recv() {
            self.update(stats_update);
        }
        // After processing all stats, print the summary report or don't if in view mode
        if self.config.view().is_some() {
            // Avoid printing the report in the middle of a view
            log::info!("View active, skipping report summary printout.")
        } else {
            self.print();
        }
    }

    fn update(&mut self, stat: StatType) {
        //self.print();
        match stat {
            StatType::Error(msg) => {
                if self.fatal_error.is_some() {
                    // Stop processing any error messages
                    log::trace!("Fatal error already seen, ignoring error: {}", msg);
                    return;
                }
                if self.max_tolerate_errors == 0 {
                    error!("{msg}");
                    self.non_atomic_total_errors += 1;
                } else {
                    let prv_err_cnt = self.total_errors.load(std::sync::atomic::Ordering::SeqCst);
                    if prv_err_cnt >= self.max_tolerate_errors {
                        return;
                    }
                    error!("{msg}");
                    let prv_err_cnt = self
                        .total_errors
                        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    log::info!("Error count: {}", prv_err_cnt + 1);
                    if prv_err_cnt + 1 == self.max_tolerate_errors {
                        log::info!("Errors reached maximum tolerated errors, exiting...");
                        self.end_processing_flag
                            .store(true, std::sync::atomic::Ordering::SeqCst);
                    }
                }
            }
            StatType::RDHsSeen(val) => self.rdhs_seen += val as u64,
            StatType::RDHsFiltered(val) => self.rdhs_filtered += val as u64,
            StatType::PayloadSize(size) => self.payload_size += size as u64,
            StatType::LinksObserved(val) => self.links_observed.push(val),
            StatType::RdhVersion(version) => self.rdh_version = version,
            StatType::DataFormat(version) => {
                if !self.data_formats_observed.contains(&version) {
                    self.data_formats_observed.push(version);
                }
            }
            StatType::HBFsSeen(val) => self.hbfs_seen += val,
            StatType::Fatal(err) => {
                if self.fatal_error.is_some() {
                    // Stop processing any error messages
                    log::trace!("Fatal error already seen, ignoring error: {}", err);
                    return;
                }
                self.end_processing_flag
                    .store(true, std::sync::atomic::Ordering::SeqCst);
                log::error!("FATAL: {err}\nShutting down...");
                self.fatal_error = Some(err);
            }
            StatType::LayerStaveSeen { layer, stave } => {
                // Only add if not already seen
                if !self.layers_staves_seen.contains(&(layer, stave)) {
                    self.layers_staves_seen.push((layer, stave));
                }
            }
            StatType::RunTriggerType((raw_trigger_type, trigger_type_str)) => {
                let (raw_val, string_descr) = self.run_trigger_type.to_owned();
                if raw_val == 0 && string_descr.is_empty() {
                    log::debug!(
                        "Run trigger type determined to be {raw_trigger_type:#0x}: {trigger_type_str}"
                    );
                    self.run_trigger_type = (raw_trigger_type, trigger_type_str);
                } else {
                    // Error happened, the run trigger type should only be reported once
                    let error = String::from("Run trigger type reported more than once!");
                    self.end_processing_flag
                        .store(true, std::sync::atomic::Ordering::SeqCst);
                    log::error!("FATAL: {error}\nShutting down...");
                    self.fatal_error = Some(error);
                }
            }
            StatType::SystemId(sys_id) => self.system_id_observed = Some(sys_id),
        }
    }

    /// Builds and prints the report
    fn print(&self) {
        let mut report = Report::new(self.processing_time.elapsed());
        if let Some(err) = &self.fatal_error {
            report.add_fatal_error(err.clone());
        }
        // Add global stats
        if self.max_tolerate_errors == 0 {
            report.add_stat(StatSummary::new(
                "Total Errors".to_string(),
                self.non_atomic_total_errors.to_string(),
                None,
            ));
        } else {
            report.add_stat(StatSummary::new(
                "Total Errors".to_string(),
                self.total_errors
                    .load(std::sync::atomic::Ordering::SeqCst)
                    .to_string(),
                None,
            ));
        }
        let trigger_type_raw = self.run_trigger_type.0.to_owned();
        report.add_stat(StatSummary {
            statistic: "Run Trigger Type".to_string(),
            value: format!("{trigger_type_raw:#02X}"),
            notes: self.run_trigger_type.1.to_owned(),
        });
        report.add_stat(StatSummary::new(
            "Total RDHs".to_string(),
            self.rdhs_seen.to_string(),
            None,
        ));
        report.add_stat(StatSummary::new(
            "Links observed during scan".to_string(),
            format_links_observed(self.links_observed.clone()),
            None,
        ));

        if self.config.filter_link().is_none() {
            // If no filtering, the HBFs seen is from the total RDHs
            report.add_stat(StatSummary::new(
                "Total HBFs".to_string(),
                self.hbfs_seen.to_string(),
                None,
            ));

            // Check if the observed system ID is ITS
            if matches!(self.system_id_observed, Some(SystemId::ITS)) {
                // If no filtering, the layers and staves seen is from the total RDHs
                report.add_stat(StatSummary::new(
                    "Layers and Staves seen".to_string(),
                    format_layers_and_staves(self.layers_staves_seen.clone()),
                    None,
                ));
            }

            // If no filtering, the payload size seen is from the total RDHs
            report.add_stat(StatSummary::new(
                "Total Payload Size".to_string(),
                format_payload(self.payload_size),
                None,
            ));
        } else {
            let filtered_stats: Vec<StatSummary> = self.add_filtered_stats();
            report.add_filter_stats(tabled::Table::new(filtered_stats));
        }

        // Add detected attributes
        report.add_detected_attribute("RDH Version".to_string(), self.rdh_version.to_string());
        let observed_data_formats_string =
            self.check_and_format_observed_data_formats(self.data_formats_observed.clone());
        report.add_detected_attribute("Data Format".to_string(), observed_data_formats_string);
        report.add_detected_attribute(
            "System ID".to_string(),
            self.system_id_observed.unwrap_or(SystemId::TST).to_string(), // Default to TST for unit tests where no RDHs are seen
        );

        report.print();
    }

    fn check_and_format_observed_data_formats(&self, observed_data_formats: Vec<u8>) -> String {
        let mut observed_data_formats = observed_data_formats;
        if observed_data_formats.len() > 1 {
            observed_data_formats.sort();
            log::error!(
                "Multiple data formats observed: {:?}",
                observed_data_formats
            );
        }
        observed_data_formats
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    }

    /// Helper function that builds a vector of the stats associated with the filtered data
    fn add_filtered_stats(&self) -> Vec<StatSummary> {
        let mut filtered_stats: Vec<StatSummary> = Vec::new();
        filtered_stats.push(StatSummary::new(
            "RDHs".to_string(),
            self.rdhs_filtered.to_string(),
            None,
        ));
        filtered_stats.push(StatSummary::new(
            "Total Payload Size".to_string(),
            format_payload(self.payload_size),
            None,
        ));

        let filtered_links = summerize_filtered_links(
            self.config.filter_link().unwrap(),
            self.links_observed.clone(),
        );
        filtered_stats.push(filtered_links);

        filtered_stats
    }
}

/// Format and add payload size seen/loaded
fn format_payload(payload_size: u64) -> String {
    match payload_size {
        0..=1024 => format!("{} B", payload_size),
        1025..=1048576 => {
            format!("{:.3} KiB", payload_size as f64 / 1024_f64)
        }
        1048577..=1073741824 => {
            format!("{:.3} MiB", payload_size as f64 / 1048576_f64)
        }
        _ => format!("{:.3} GiB", payload_size as f64 / 1073741824_f64),
    }
}

/// Sort and format links observed
fn format_links_observed(links_observed: Vec<u8>) -> String {
    let mut observed_links = links_observed;
    observed_links.sort();
    observed_links
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(", ")
}

/// Sort and format layers and staves seen
fn format_layers_and_staves(layers_staves_seen: Vec<(u8, u8)>) -> String {
    let mut layers_staves_seen = layers_staves_seen;
    layers_staves_seen.sort();
    layers_staves_seen
        .iter()
        .enumerate()
        .map(|(i, (layer, stave))| {
            if i > 0 && i % 7 == 0 {
                format!("L{layer}_{stave}\n")
            } else {
                format!("L{layer}_{stave} ")
            }
        })
        .collect::<Vec<String>>()
        .join("")
}

/// Helper functions to format the summary
fn summerize_filtered_links(link_to_filter: u8, links_observed: Vec<u8>) -> StatSummary {
    let mut filtered_links_stat = StatSummary::new("Link ID".to_string(), "".to_string(), None);
    // Format links that were filtered, separated by commas
    if links_observed.contains(&link_to_filter) {
        filtered_links_stat.value = link_to_filter.to_string();
    } else {
        filtered_links_stat.value = "<<none>>".to_string();
        filtered_links_stat.notes = format!("not found: {link_to_filter}");
    }
    filtered_links_stat
}
