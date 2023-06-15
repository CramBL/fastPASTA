//! Contains the [StatsController] that collects stats and reports errors.
//! It also controls the stop flag, which can be used to stop the program if a fatal error occurs, or if the config contains a max number of errors to tolerate.
//! Finally when the event loop breaks (at the end of execution), it will print a summary of the stats collected, using the Report struct.

use super::{
    lib::{StatType, SystemId},
    rdh_stats::RdhStats,
    stat_summerize_utils::{
        summerize_filtered_fee_ids, summerize_filtered_its_layer_staves, summerize_filtered_links,
    },
};
use crate::{
    stats::stats_report::report::{Report, StatSummary},
    util::lib::{Config, DataOutputMode, FilterTarget},
    words::{
        self,
        its::{layer_from_feeid, stave_number_from_feeid},
        lib::RDH_CRU_SIZE_BYTES,
    },
};
use owo_colors::OwoColorize;
use std::sync::{atomic::AtomicBool, Arc};

/// The StatsController receives stats and builds a summary report that is printed at the end of execution.
pub struct StatsController<C: Config + 'static> {
    rdh_stats: RdhStats,
    /// Time from [StatsController] is instantiated, to all data processing threads disconnected their [StatType] producer channel.
    pub processing_time: std::time::Instant,
    config: &'static C,
    total_errors: u64,
    reported_errors: Vec<String>,
    max_tolerate_errors: u32,
    // The channel where stats are received from other threads.
    recv_stats_channel: flume::Receiver<StatType>,
    // The channel stats are sent through, stored so that a clone of the channel can be returned easily
    // Has to be an option so that it can be set to None when the event loop starts.
    // Once run is called no producers that don't already have a channel to send stats through, will be able to get one.
    // This is because the event loop breaks when all sender channels are dropped, and if the StatsController keeps a reference to the channel, it will cause a deadlock.
    send_stats_channel: Option<flume::Sender<StatType>>,
    end_processing_flag: Arc<AtomicBool>,
    fatal_error: Option<String>,
    staves_with_errors: Vec<(u8, u8)>,
    any_errors_flag: Arc<AtomicBool>,
    error_codes: Vec<u8>,
}
impl<C: Config + 'static> StatsController<C> {
    /// Creates a new [StatsController] from a [Config], a [flume::Receiver] for [StatType], and a [std::sync::Arc] of an [AtomicBool] that is used to signal to other threads to exit if a fatal error occurs.
    pub fn new(global_config: &'static C) -> Self {
        let (send_stats_channel, recv_stats_channel): (
            flume::Sender<StatType>,
            flume::Receiver<StatType>,
        ) = flume::unbounded();
        StatsController {
            rdh_stats: RdhStats::default(),
            config: global_config,
            processing_time: std::time::Instant::now(),
            total_errors: 0,
            reported_errors: Vec::new(),
            max_tolerate_errors: global_config.max_tolerate_errors(),
            recv_stats_channel,
            send_stats_channel: Some(send_stats_channel),
            end_processing_flag: Arc::new(AtomicBool::new(false)),
            fatal_error: None,
            staves_with_errors: Vec::new(),
            any_errors_flag: Arc::new(AtomicBool::new(false)),
            error_codes: Vec::new(),
        }
    }

    /// Returns a clone of the channel that is used to send stats to the StatsController.
    pub fn send_channel(&self) -> flume::Sender<StatType> {
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

    /// Returns a cloned reference to the any errors flag
    ///
    /// The flag is set if there's any errors in the input data at end of processing.
    pub fn any_errors_flag(&self) -> Arc<AtomicBool> {
        self.any_errors_flag.clone()
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
        if self.config.view().is_some() || self.config.output_mode() == DataOutputMode::Stdout {
            // Avoid printing the report in the middle of a view, or if output is being redirected
            log::info!("View active or output is being piped, skipping report summary printout.")
        } else {
            self.process_error_messages();

            // Print the summary report if any RDHs were seen. If not, it's likely that an early error occurred and no data was processed.
            if self.rdh_stats.rdhs_seen > 0 {
                self.print();
            }
        }
        if self.total_errors > 0 {
            self.any_errors_flag
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }

    fn update(&mut self, stat: StatType) {
        match stat {
            StatType::Error(msg) => {
                if self.fatal_error.is_some() {
                    // Stop processing any error messages
                    log::trace!("Fatal error already seen, ignoring error: {msg}");
                    return;
                }

                self.reported_errors.push(msg);
                self.total_errors += 1;

                if self.max_tolerate_errors > 0 {
                    log::trace!("Error count: {}", self.total_errors);
                    if self.total_errors == self.max_tolerate_errors as u64 {
                        log::trace!("Errors reached maximum tolerated errors, exiting...");
                        self.end_processing_flag
                            .store(true, std::sync::atomic::Ordering::SeqCst);
                    }
                }
            }
            StatType::RDHsSeen(val) => self.rdh_stats.rdhs_seen += val as u64,
            StatType::RDHsFiltered(val) => self.rdh_stats.rdhs_filtered += val as u64,
            StatType::PayloadSize(size) => self.rdh_stats.payload_size += size as u64,
            StatType::LinksObserved(val) => self.rdh_stats.record_link(val),
            StatType::RdhVersion(version) => self.rdh_stats.record_rdh_version(version),
            StatType::DataFormat(version) => {
                self.rdh_stats.record_data_format(version);
            }
            StatType::HBFsSeen(val) => self.rdh_stats.hbfs_seen += val,
            StatType::Fatal(err) => {
                if self.fatal_error.is_some() {
                    // Stop processing any error messages
                    log::trace!("Fatal error already seen, ignoring error: {err}");
                    return;
                }
                self.end_processing_flag
                    .store(true, std::sync::atomic::Ordering::SeqCst);
                log::error!("FATAL: {err}\nShutting down...");
                self.fatal_error = Some(err);
            }
            StatType::LayerStaveSeen { layer, stave } => {
                self.rdh_stats.record_layer_stave_seen((layer, stave));
            }
            StatType::RunTriggerType((raw_trigger_type, trigger_type_str)) => {
                log::debug!(
                    "Run trigger type determined to be {raw_trigger_type:#0x}: {trigger_type_str}"
                );
                self.rdh_stats
                    .record_run_trigger_type((raw_trigger_type, trigger_type_str));
            }
            StatType::SystemId(sys_id) => self.rdh_stats.record_system_id(sys_id),
            StatType::FeeId(id) => {
                self.rdh_stats.record_fee_observed(id);
            }
        }
    }

    fn process_error_messages(&mut self) {
        self.sort_errors_by_memory_address();
        self.extract_unique_error_codes();
        if matches!(self.rdh_stats.system_id(), Some(SystemId::ITS)) {
            self.check_error_for_stave_id();
        }
        if !self.config.mute_errors() {
            // Print the errors, limited if there's a max error limit set
            if self.max_tolerate_errors > 0 {
                self.reported_errors
                    .drain(..)
                    .take(self.max_tolerate_errors as usize)
                    .for_each(|e| {
                        super::display_error(&e);
                    });
            } else {
                self.reported_errors.drain(..).for_each(|e| {
                    super::display_error(&e);
                });
            }
        }
    }

    fn sort_errors_by_memory_address(&mut self) {
        // Regex to extract the memory address from the error message
        let re = regex::Regex::new(r"0x(?P<mem_pos>[0-9a-fA-F]+):").unwrap();
        // Sort the errors by memory address
        if !self.reported_errors.is_empty() {
            self.reported_errors.sort_by_key(|e| {
                let addr = re
                    .captures(e)
                    .unwrap_or_else(|| panic!("Error parsing memory address from error msg: {e}"));
                u64::from_str_radix(&addr["mem_pos"], 16).expect("Error parsing memory address")
            });
        }
    }

    fn extract_unique_error_codes(&mut self) {
        let re = regex::Regex::new(r"0x.*: \[E(?P<err_code>[0-9]{2})\]").unwrap();
        self.reported_errors.iter().for_each(|err_msg| {
            let err_code_match: regex::Captures = re
                .captures(err_msg)
                .unwrap_or_else(|| panic!("Error parsing error code from error msg: {err_msg}"));

            let err_code = err_code_match["err_code"].parse::<u8>().unwrap();
            if !self.error_codes.contains(&err_code) {
                self.error_codes.push(err_code);
            }
        });
    }

    fn check_error_for_stave_id(&mut self) {
        let re: regex::Regex =
            regex::Regex::new("FEE(?:.|)ID:(?P<fee_id>[1-9][0-9]{0,4})").unwrap();

        self.reported_errors.iter().for_each(|err_msg| {
            let fee_id_match: Option<regex::Captures> = re.captures(err_msg);
            if let Some(id_capture) = fee_id_match {
                let fee_id = id_capture["fee_id"].parse::<u16>().unwrap();
                let layer = layer_from_feeid(fee_id);
                let stave = stave_number_from_feeid(fee_id);

                let stave_with_error = self
                    .rdh_stats
                    .layer_staves_as_slice()
                    .iter()
                    .find(|(l, s)| *l == layer && *s == stave)
                    .expect(
                        "FEE ID found in error message that does not match any layer/stave seen",
                    );

                if !self.staves_with_errors.contains(stave_with_error) {
                    self.staves_with_errors.push(*stave_with_error);
                }
            }
        });
    }

    /// Builds and prints the report
    fn print(&mut self) {
        let mut report = Report::new(self.processing_time.elapsed());
        // Add fatal error if any
        if let Some(err) = &self.fatal_error {
            report.add_fatal_error(err.clone());
        }
        // Add global stats
        self.add_global_stats_to_report(&mut report);

        if !self.config.filter_enabled() {
            // Check if the observed system ID is ITS
            if matches!(self.rdh_stats.system_id(), Some(SystemId::ITS)) {
                // If no filtering, the layers and staves seen is from the total RDHs
                report.add_stat(summerize_layers_staves_seen(
                    self.rdh_stats.layer_staves_as_slice(),
                    &self.staves_with_errors,
                ));
            }
            // If no filtering, the HBFs seen is from the total RDHs
            report.add_stat(StatSummary::new(
                "Total HBFs".to_string(),
                self.rdh_stats.hbfs_seen.to_string(),
                None,
            ));

            // If no filtering, the payload size seen is from the total RDHs
            report.add_stat(summerize_data_size(
                self.rdh_stats.rdhs_seen,
                self.rdh_stats.payload_size,
            ));
        } else {
            let filtered_stats: Vec<StatSummary> = self.add_filtered_stats();
            report.add_filter_stats(tabled::Table::new(filtered_stats));
        }

        // Add detected attributes
        add_detected_attributes_to_report(&mut report, &self.rdh_stats);

        report.print();
    }

    fn add_global_stats_to_report(&mut self, report: &mut Report) {
        if self.total_errors == 0 {
            report.add_stat(StatSummary::new(
                "Total Errors".green().to_string(),
                self.total_errors.green().to_string(),
                None,
            ));
        } else {
            report.add_stat(StatSummary::new(
                "Total Errors".red().to_string(),
                self.total_errors.red().to_string(),
                Some(
                    self.error_codes
                        .iter()
                        .enumerate()
                        .map(|(i, code)| {
                            if i > 0 && i % 5 == 0 {
                                format!("E{code}\n")
                            } else {
                                format!("E{code} ")
                            }
                        })
                        .collect(),
                ),
            ));
        }

        let (trigger_type_raw, trigger_type_str) = self.rdh_stats.run_trigger_type();
        report.add_stat(StatSummary {
            statistic: "Run Trigger Type".to_string(),
            value: format!("{trigger_type_raw:#02X}"),
            notes: trigger_type_str,
        });
        report.add_stat(StatSummary::new(
            "Total RDHs".to_string(),
            self.rdh_stats.rdhs_seen.to_string(),
            None,
        ));
        self.rdh_stats.sort_links_observed();
        report.add_stat(StatSummary::new(
            "Links observed".to_string(),
            format_links_observed(self.rdh_stats.links_as_slice()),
            None,
        ));
        report.add_stat(StatSummary::new(
            "FEE IDs seen".to_string(),
            format_fee_ids(self.rdh_stats.fee_ids_as_slice()),
            None,
        ));
    }

    /// Helper function that builds a vector of the stats associated with the filtered data
    fn add_filtered_stats(&mut self) -> Vec<StatSummary> {
        let mut filtered_stats: Vec<StatSummary> = Vec::new();
        filtered_stats.push(StatSummary::new(
            "RDHs".to_string(),
            self.rdh_stats.rdhs_filtered.to_string(),
            None,
        ));
        // If filtering, the HBFs seen is from the filtered RDHs
        filtered_stats.push(StatSummary::new(
            "HBFs".to_string(),
            self.rdh_stats.hbfs_seen.to_string(),
            None,
        ));

        filtered_stats.push(summerize_data_size(
            self.rdh_stats.rdhs_filtered,
            self.rdh_stats.payload_size,
        ));

        if let Some(filter_target) = self.config.filter_target() {
            let filtered_target = match filter_target {
                FilterTarget::Link(link_id) => {
                    summerize_filtered_links(link_id, self.rdh_stats.links_as_slice())
                }
                FilterTarget::Fee(fee_id) => {
                    summerize_filtered_fee_ids(fee_id, self.rdh_stats.fee_ids_as_slice())
                }
                FilterTarget::ItsLayerStave(fee_id_no_link) => summerize_filtered_its_layer_staves(
                    fee_id_no_link,
                    self.rdh_stats.layer_staves_as_slice(),
                ),
            };
            filtered_stats.push(filtered_target);
        }

        if self
            .config
            .filter_target()
            .is_some_and(|target| !matches!(target, FilterTarget::ItsLayerStave(_)))
        {
            // Check if the observed system ID is ITS
            if matches!(self.rdh_stats.system_id(), Some(SystemId::ITS)) {
                // If no filtering, the layers and staves seen is from the total RDHs
                filtered_stats.push(summerize_layers_staves_seen(
                    self.rdh_stats.layer_staves_as_slice(),
                    self.staves_with_errors.as_slice(),
                ));
            }
        }

        filtered_stats
    }
}

fn add_detected_attributes_to_report(report: &mut Report, rdh_stats: &RdhStats) {
    report.add_detected_attribute(
        "RDH Version".to_string(),
        rdh_stats.rdh_version().to_string(),
    );

    report.add_detected_attribute(
        "Data Format".to_string(),
        rdh_stats.data_format().to_string(),
    );
    report.add_detected_attribute(
        "System ID".to_string(),
        // If no system ID is found, something is wrong, set it to "none" in red.
        match rdh_stats.system_id() {
            Some(sys_id) => sys_id.to_string(),
            None => String::from("none").red().to_string(),
        }, // Default to TST for unit tests where no RDHs are seen
    );
}

fn summerize_layers_staves_seen(
    layers_staves_seen: &[(u8, u8)],
    staves_with_errors: &[(u8, u8)],
) -> StatSummary {
    StatSummary::new(
        "Layers/Staves".to_string(),
        format_layers_and_staves(layers_staves_seen.to_owned(), staves_with_errors.to_owned()),
        None,
    )
}

fn summerize_data_size(rdh_count: u64, payload_size: u64) -> StatSummary {
    let rdh_data_size = rdh_count * RDH_CRU_SIZE_BYTES as u64;
    if rdh_data_size == 0 {
        StatSummary::new("Data size".to_string(), format_data_size(0), None)
    } else {
        StatSummary::new(
            "Data size".to_string(),
            format_data_size(rdh_data_size + payload_size),
            Some(format!(
                "RDHs:     {rdhs_size}\nPayloads: {payloads_size}",
                rdhs_size = format_data_size(rdh_data_size),
                payloads_size = format_data_size(payload_size)
            )),
        )
    }
}

/// Format a size in bytes to human readable.
fn format_data_size(size_bytes: u64) -> String {
    match size_bytes {
        0..=1024 => format!("{} B", size_bytes),
        1025..=1048576 => {
            format!("{:.2} KiB", size_bytes as f64 / 1024_f64)
        }
        1048577..=1073741824 => {
            format!("{:.2} MiB", size_bytes as f64 / 1048576_f64)
        }
        _ => format!("{:.2} GiB", size_bytes as f64 / 1073741824_f64),
    }
}

/// Sort and format links observed
fn format_links_observed(links_observed: &[u8]) -> String {
    links_observed
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(", ")
}

/// Sort and format layers and staves seen
fn format_layers_and_staves(
    mut layers_staves_seen: Vec<(u8, u8)>,
    mut layers_stave_with_errors: Vec<(u8, u8)>,
) -> String {
    if layers_staves_seen.is_empty() {
        return "none".red().to_string();
    }
    layers_staves_seen.sort();
    layers_stave_with_errors.sort();

    layers_staves_seen
        .iter()
        .enumerate()
        .map(|(i, (layer, stave))| {
            if i > 0 && i % 7 == 0 {
                if layers_stave_with_errors.contains(&(*layer, *stave)) {
                    format!("L{layer}_{stave}\n").red().to_string()
                } else {
                    format!("L{layer}_{stave}\n")
                }
            } else if layers_stave_with_errors.contains(&(*layer, *stave)) {
                format!("L{layer}_{stave} ").red().to_string()
            } else {
                format!("L{layer}_{stave} ")
            }
        })
        .collect::<Vec<String>>()
        .join("")
}

fn format_fee_ids(fee_ids_seen: &[u16]) -> String {
    if fee_ids_seen.is_empty() {
        return "none".red().to_string();
    }
    let mut fee_ids_seen = fee_ids_seen.to_owned();
    fee_ids_seen.sort();
    fee_ids_seen
        .iter()
        .enumerate()
        .map(|(i, id)| {
            if i > 0 && i % 5 == 0 {
                format!("{id}\n")
            } else {
                format!("{id} ")
            }
        })
        .collect::<Vec<String>>()
        .join(", ")
}
