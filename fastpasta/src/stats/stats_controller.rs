//! Contains the [StatsController] that collects stats and reports errors.
//! It also controls the stop flag, which can be used to stop the program if a fatal error occurs, or if the config contains a max number of errors to tolerate.
//! Finally when the event loop breaks (at the end of execution), it will print a summary of the stats collected, using the Report struct.

use super::super::StatType;

use super::error_stats::ErrorStats;
use super::lib;
use super::rdh_stats::RdhStats;
use super::stat_format_utils::format_error_codes;
use super::stat_format_utils::format_fee_ids;
use super::stat_format_utils::format_links_observed;
use super::stat_summerize_utils::summerize_data_size;
use super::stat_summerize_utils::summerize_filtered_fee_ids;
use super::stat_summerize_utils::summerize_filtered_its_layer_staves;
use super::stat_summerize_utils::summerize_filtered_links;
use super::stat_summerize_utils::summerize_layers_staves_seen;
use super::stats_validation::validate_custom_stats;
use super::SystemId;
use crate::config::prelude::*;
use crate::stats::stats_report::report::Report;
use crate::stats::stats_report::report::StatSummary;
use alice_daq_protocol_reader::prelude::FilterTarget;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// The StatsController receives stats and builds a summary report that is printed at the end of execution.
pub struct StatsController<C: Config + 'static> {
    rdh_stats: RdhStats,
    error_stats: ErrorStats,
    /// Time from [StatsController] is instantiated, to all data processing threads disconnected their [StatType] producer channel.
    pub processing_time: std::time::Instant,
    config: &'static C,
    max_tolerate_errors: u32,
    // The channel where stats are received from other threads.
    recv_stats_channel: flume::Receiver<StatType>,
    // The channel stats are sent through, stored so that a clone of the channel can be returned easily
    // Has to be an option so that it can be set to None when the event loop starts.
    // Once run is called no producers that don't already have a channel to send stats through, will be able to get one.
    // This is because the event loop breaks when all sender channels are dropped, and if the StatsController keeps a reference to the channel, it will cause a deadlock.
    send_stats_channel: Option<flume::Sender<StatType>>,
    end_processing_flag: Arc<AtomicBool>,
    any_errors_flag: Arc<AtomicBool>,
    spinner: Option<ProgressBar>,
    spinner_message: String,
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
            error_stats: ErrorStats::default(),
            config: global_config,
            processing_time: std::time::Instant::now(),
            max_tolerate_errors: global_config.max_tolerate_errors(),
            recv_stats_channel,
            send_stats_channel: Some(send_stats_channel),
            end_processing_flag: Arc::new(AtomicBool::new(false)),
            any_errors_flag: Arc::new(AtomicBool::new(false)),
            spinner: if global_config.view().is_some() {
                None
            } else {
                Some(new_styled_spinner())
            },
            spinner_message: String::new(),
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

        if self.config.custom_checks_enabled() {
            if let Err(e) = validate_custom_stats(self.config, &self.rdh_stats) {
                e.into_iter().for_each(|e| {
                    self.error_stats.add_custom_check_error(e);
                });
            }
        }

        // After processing all stats, print the summary report or don't if in view mode
        if self.config.view().is_some() || self.config.output_mode() == DataOutputMode::Stdout {
            // Avoid printing the report in the middle of a view, or if output is being redirected
            log::info!("View active or output is being piped, skipping report summary printout.")
        } else {
            if self.error_stats.total_errors() > 0 {
                self.process_error_messages();
            }

            // Print the summary report if any RDHs were seen. If not, it's likely that an early error occurred and no data was processed.
            if self.rdh_stats.rdhs_seen() > 0 {
                // New spinner/progress bar
                self.new_spinner_with_prefix("Generating report".to_string());
                self.print();
            }
        }
        if self.error_stats.total_errors() > 0 {
            self.any_errors_flag
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }

    fn update(&mut self, stat: StatType) {
        match stat {
            StatType::Error(msg) => {
                if self.error_stats.is_fatal_error() {
                    // Stop processing any error messages
                    log::trace!("Fatal error already seen, ignoring error: {msg}");
                    return;
                }

                self.error_stats.add_reported_error(msg);
                self.set_spinner_msg(
                    format!(
                        "{err_cnt} Errors in data!",
                        err_cnt = self.error_stats.total_errors()
                    )
                    .red()
                    .to_string(),
                );

                if self.max_tolerate_errors > 0 {
                    log::trace!("Error count: {}", self.error_stats.total_errors());
                    if self.error_stats.total_errors() == self.max_tolerate_errors as u64 {
                        log::trace!("Errors reached maximum tolerated errors, exiting...");
                        self.end_processing_flag
                            .store(true, std::sync::atomic::Ordering::SeqCst);
                    }
                }
            }
            StatType::RDHSeen(val) => self.rdh_stats.add_rdhs_seen(val),
            StatType::RDHFiltered(val) => self.rdh_stats.add_rdhs_filtered(val),
            StatType::PayloadSize(size) => self.rdh_stats.add_payload_size(size as u64),
            StatType::LinksObserved(val) => self.rdh_stats.record_link(val),
            StatType::RdhVersion(version) => self.rdh_stats.record_rdh_version(version),
            StatType::DataFormat(version) => {
                self.rdh_stats.record_data_format(version);
            }
            StatType::HBFSeen => {
                self.rdh_stats.incr_hbf_seen();
                if self.spinner.is_some() {
                    self.spinner.as_mut().unwrap().set_prefix(format!(
                        "Analyzing {hbfs} HBFs",
                        hbfs = self.rdh_stats.hbfs_seen()
                    ))
                };
            }
            StatType::Fatal(err) => {
                if self.error_stats.is_fatal_error() {
                    // Stop processing any error messages
                    log::trace!("Fatal error already seen, ignoring error: {err}");
                    return;
                }
                self.end_processing_flag
                    .store(true, std::sync::atomic::Ordering::SeqCst);
                log::error!("FATAL: {err}\nShutting down...");
                self.error_stats.add_fatal_error(err);
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
            StatType::TriggerType(val) => self.rdh_stats.record_trigger_type(val),
        }
    }

    fn process_error_messages(&mut self) {
        // New spinner/progress bar
        self.new_spinner_with_prefix(
            format!(
                "Processing {err_count} error messages",
                err_count = self.error_stats.total_errors()
            )
            .yellow()
            .to_string(),
        );
        self.error_stats.finalize_stats();
        if matches!(self.rdh_stats.system_id(), Some(SystemId::ITS)) {
            self.error_stats
                .check_errors_for_stave_id(self.rdh_stats.layer_staves_as_slice());
        }
        if !self.config.mute_errors() {
            // Print the errors, limited if there's a max error limit set
            if self.max_tolerate_errors > 0 {
                self.error_stats
                    .consume_reported_errors()
                    .drain(..)
                    .take(self.max_tolerate_errors as usize)
                    .for_each(|e| {
                        lib::display_error(&e);
                    });
            } else {
                self.error_stats
                    .consume_reported_errors()
                    .drain(..)
                    .for_each(|e| {
                        lib::display_error(&e);
                    });
            }
        }
    }

    /// Builds and prints the report
    fn print(&mut self) {
        let mut report = Report::new(self.processing_time.elapsed());
        // Add fatal error if any
        if self.error_stats.is_fatal_error() {
            report.add_fatal_error(self.error_stats.take_fatal_error());
        }
        // Add global stats
        self.add_global_stats_to_report(&mut report);

        if !self.config.filter_enabled() {
            // Check if the observed system ID is ITS
            if matches!(self.rdh_stats.system_id(), Some(SystemId::ITS)) {
                // If no filtering, the layers and staves seen is from the total RDHs
                report.add_stat(summerize_layers_staves_seen(
                    self.rdh_stats.layer_staves_as_slice(),
                    self.error_stats.staves_with_errors_as_slice(),
                ));
            }
            // If no filtering, the HBFs seen is from the total RDHs
            report.add_stat(StatSummary::new(
                "Total HBFs".to_string(),
                self.rdh_stats.hbfs_seen().to_string(),
                None,
            ));

            // If no filtering, the payload size seen is from the total RDHs
            report.add_stat(summerize_data_size(
                self.rdh_stats.rdhs_seen(),
                self.rdh_stats.payload_size(),
            ));
        } else {
            let filtered_stats: Vec<StatSummary> = self.add_filtered_stats();
            report.add_filter_stats(tabled::Table::new(filtered_stats));
        }

        // Add detected attributes
        add_detected_attributes_to_report(&mut report, &self.rdh_stats);

        self.append_spinner_msg("... completed");
        if self.spinner.is_some() {
            self.spinner.as_mut().unwrap().abandon();
        }
        report.print();
    }

    fn add_global_stats_to_report(&mut self, report: &mut Report) {
        if self.error_stats.total_errors() == 0 {
            report.add_stat(StatSummary::new(
                "Total Errors".green().to_string(),
                self.error_stats.total_errors().green().to_string(),
                None,
            ));
        } else {
            report.add_stat(StatSummary::new(
                "Total Errors".red().to_string(),
                self.error_stats.total_errors().red().to_string(),
                Some(format_error_codes(
                    self.error_stats.unique_error_codes_as_slice(),
                )),
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
            self.rdh_stats.rdhs_seen().to_string(),
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
            self.rdh_stats.rdhs_filtered().to_string(),
            None,
        ));
        // If filtering, the HBFs seen is from the filtered RDHs
        filtered_stats.push(StatSummary::new(
            "HBFs".to_string(),
            self.rdh_stats.hbfs_seen().to_string(),
            None,
        ));

        filtered_stats.push(summerize_data_size(
            self.rdh_stats.rdhs_filtered(),
            self.rdh_stats.payload_size(),
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
                    self.error_stats.staves_with_errors_as_slice(),
                ));
            }
        }

        filtered_stats
    }

    /// Add completed message to current spinner and abandon it
    /// Replace it with new spinner with an empty message
    /// Set the new spinners prefix message
    fn new_spinner_with_prefix(&mut self, prefix: String) {
        if self.spinner.is_some() {
            self.append_spinner_msg("... completed");
            self.spinner.as_mut().unwrap().abandon();
            self.spinner = Some(new_styled_spinner());
            self.spinner_message = "".to_string();
            self.spinner.as_mut().unwrap().set_prefix(prefix);
        }
    }

    fn set_spinner_msg(&mut self, new_msg: String) {
        if self.spinner.is_some() {
            self.spinner_message = new_msg;
            self.spinner
                .as_mut()
                .unwrap()
                .set_message(self.spinner_message.clone());
        }
    }

    fn append_spinner_msg(&mut self, to_append: &str) {
        if self.spinner.is_some() {
            self.spinner_message = self.spinner_message.clone() + to_append + " ";
            self.spinner
                .as_mut()
                .unwrap()
                .set_message(self.spinner_message.clone());
        }
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

fn new_styled_spinner() -> ProgressBar {
    let spinner_style =
        ProgressStyle::with_template("{spinner} [ {prefix:.bold.blue} ] {wide_msg}")
            .unwrap()
            .tick_strings(&[
                "▹▹▹▹▹",
                "▸▹▹▹▹",
                "▹▸▹▹▹",
                "▹▹▸▹▹",
                "▹▹▹▸▹",
                "▹▹▹▹▸",
                "▪▪▪▪▪",
            ]);
    let pb = ProgressBar::new_spinner();
    pb.set_style(spinner_style);
    pb.enable_steady_tick(std::time::Duration::from_millis(120));
    pb
}
