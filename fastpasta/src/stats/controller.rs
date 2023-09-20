//! Contains the [Controller] that collects stats and reports errors.
//! It also controls the stop flag, which can be used to stop the program if a fatal error occurs, or if the config contains a max number of errors to tolerate.
//! Finally when the event loop breaks (at the end of execution), it will print a summary of the stats collected, using the Report struct.

use super::super::StatType;

use super::lib;
use super::stats_collector::its_stats::alpide_stats::AlpideStats;
use super::stats_collector::rdh_stats::RdhStats;
use super::stats_collector::StatsCollector;
use super::stats_report::stat_format_utils::format_error_codes;
use super::stats_report::stat_format_utils::format_fee_ids;
use super::stats_report::stat_format_utils::format_links_observed;
use super::stats_report::stat_summerize_utils::summerize_data_size;
use super::stats_report::stat_summerize_utils::summerize_filtered_fee_ids;
use super::stats_report::stat_summerize_utils::summerize_filtered_its_layer_staves;
use super::stats_report::stat_summerize_utils::summerize_filtered_links;
use super::stats_report::stat_summerize_utils::summerize_layers_staves_seen;
use super::SystemId;
use crate::config::prelude::*;
use crate::stats::stats_report::report::Report;
use crate::stats::stats_report::report::StatSummary;
use alice_protocol_reader::prelude::FilterTarget;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// The Controller receives stats and builds a summary report that is printed at the end of execution.
pub struct Controller<C: Config + 'static> {
    stats_collector: StatsCollector,
    /// Time from [Controller] is instantiated, to all data processing threads disconnected their [StatType] producer channel.
    pub processing_time: std::time::Instant,
    config: &'static C,
    max_tolerate_errors: u32,
    // The channel where stats are received from other threads.
    recv_stats_channel: flume::Receiver<StatType>,
    // The channel stats are sent through, stored so that a clone of the channel can be returned easily
    // Has to be an option so that it can be set to None when the event loop starts.
    // Once run is called no producers that don't already have a channel to send stats through, will be able to get one.
    // This is because the event loop breaks when all sender channels are dropped, and if the Controller keeps a reference to the channel, it will cause a deadlock.
    send_stats_channel: Option<flume::Sender<StatType>>,
    end_processing_flag: Arc<AtomicBool>,
    any_errors_flag: Arc<AtomicBool>,
    spinner: Option<ProgressBar>,
    spinner_message: String,
}
impl<C: Config + 'static> Controller<C> {
    /// Creates a new [Controller] from a [Config], a [flume::Receiver] for [StatType], and a [std::sync::Arc] of an [AtomicBool] that is used to signal to other threads to exit if a fatal error occurs.
    pub fn new(global_config: &'static C) -> Self {
        let (send_stats_channel, recv_stats_channel): (
            flume::Sender<StatType>,
            flume::Receiver<StatType>,
        ) = flume::unbounded();
        Controller {
            // Only collect alpide stats if alpide checks are enabled
            stats_collector: if global_config.alpide_checks_enabled() {
                StatsCollector::with_alpide_stats()
            } else {
                StatsCollector::default()
            },
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

    /// Returns a clone of the channel that is used to send stats to the Controller.
    pub fn send_channel(&self) -> flume::Sender<StatType> {
        if self.send_stats_channel.is_none() {
            log::error!("Controller send channel is none, most likely it is already running and does not accept new producers");
            panic!("Controller send channel is none, most likely it is already running and does not accept new producers");
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

    /// Starts the event loop for the Controller
    /// This function will block until the channel is closed
    pub fn run(&mut self) {
        // Set the send stats channel to none so that no new producers can be added, and so the loop breaks when all producers have dropped their channel.
        self.send_stats_channel = None;

        // While loop breaks when an error is received from the channel, which means the channel is disconnected
        while let Ok(stats_update) = self.recv_stats_channel.recv() {
            self.update(stats_update);
        }

        if self.config.custom_checks_enabled() {
            self.stats_collector.validate_custom_stats(self.config);
        }

        // After processing all stats, print the summary report or don't if in view mode
        if self.config.view().is_some() || self.config.output_mode() == DataOutputMode::Stdout {
            // Avoid printing the report in the middle of a view, or if output is being redirected
            log::info!("View active or output is being piped, skipping report summary printout.")
        } else {
            if self.stats_collector.total_errors() > 0 {
                self.process_error_messages();
            }

            // Print the summary report if any RDHs were seen. If not, it's likely that an early error occurred and no data was processed.
            if self.stats_collector.rdh_stats().rdhs_seen() > 0 {
                // New spinner/progress bar
                self.new_spinner_with_prefix("Generating report".to_string());
                self.print();
            }
        }
        if self.stats_collector.total_errors() > 0 {
            self.any_errors_flag
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }

    fn update(&mut self, stat: StatType) {
        match stat {
            StatType::Error(msg) => {
                if self.stats_collector.is_fatal_err() {
                    // Stop processing any error messages
                    log::trace!("Fatal error already seen, ignoring error: {msg}");
                    return;
                }

                self.stats_collector.collect(StatType::Error(msg));

                self.set_spinner_msg(
                    format!(
                        "{err_cnt} Errors in data!",
                        err_cnt = self.stats_collector.total_errors()
                    )
                    .red()
                    .to_string(),
                );

                if self.max_tolerate_errors > 0 {
                    log::trace!("Error count: {}", self.stats_collector.total_errors());
                    if self.stats_collector.total_errors() == self.max_tolerate_errors as u64 {
                        log::trace!("Errors reached maximum tolerated errors, exiting...");
                        self.end_processing_flag
                            .store(true, std::sync::atomic::Ordering::SeqCst);
                    }
                }
            }

            StatType::Fatal(err) => {
                if self.stats_collector.is_fatal_err() {
                    // Stop processing any error messages
                    log::trace!("Fatal error already seen, ignoring error: {err}");
                    return;
                }
                self.end_processing_flag
                    .store(true, std::sync::atomic::Ordering::SeqCst);
                log::error!("FATAL: {err}\nShutting down...");
                self.stats_collector.collect(StatType::Fatal(err));
            }
            StatType::RDHSeen(_)
            | StatType::RDHFiltered(_)
            | StatType::PayloadSize(_)
            | StatType::LinksObserved(_)
            | StatType::RdhVersion(_)
            | StatType::DataFormat(_)
            | StatType::LayerStaveSeen { .. }
            | StatType::SystemId(_)
            | StatType::FeeId(_)
            | StatType::TriggerType(_)
            | StatType::AlpideStats(_) => {
                self.stats_collector.collect(stat);
            }
            StatType::HBFSeen => {
                self.stats_collector.collect(stat);
                if self.spinner.is_some() {
                    self.spinner.as_mut().unwrap().set_prefix(format!(
                        "Analyzing {hbfs} HBFs",
                        hbfs = self.stats_collector.rdh_stats().hbfs_seen()
                    ))
                };
            }
            StatType::RunTriggerType((raw_tt, tt_str)) => {
                log::debug!("Run trigger type determined to be {raw_tt:#0x}: {tt_str}");
                self.stats_collector
                    .collect(StatType::RunTriggerType((raw_tt, tt_str)));
            }
        }
    }

    fn process_error_messages(&mut self) {
        // New spinner/progress bar
        self.new_spinner_with_prefix(
            format!(
                "Processing {err_count} error messages",
                err_count = self.stats_collector.total_errors()
            )
            .yellow()
            .to_string(),
        );
        self.stats_collector.finalize();
        self.spinner.as_mut().unwrap().abandon();

        if !self.config.mute_errors() {
            // Print the errors, limited if there's a max error limit set
            if self.max_tolerate_errors > 0 {
                self.stats_collector
                    .consume_reported_errors()
                    .drain(..)
                    .take(self.max_tolerate_errors as usize)
                    .for_each(|e| {
                        lib::display_error(&e);
                    });
            } else {
                self.stats_collector
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
        if self.stats_collector.is_fatal_err() {
            report.add_fatal_error(self.stats_collector.take_fatal_err().into_string());
        }
        // Add global stats
        self.add_global_stats_to_report(&mut report);

        if self.config.filter_enabled() {
            let filtered_stats: Vec<StatSummary> = self.add_filtered_stats();
            report.add_filter_stats(tabled::Table::new(filtered_stats));
        } else {
            // Check if the observed system ID is ITS
            if matches!(
                self.stats_collector.rdh_stats().system_id(),
                Some(SystemId::ITS)
            ) {
                // If no filtering, the layers and staves seen is from the total RDHs
                report.add_stat(summerize_layers_staves_seen(
                    self.stats_collector.rdh_stats().layer_staves_as_slice(),
                    self.stats_collector
                        .error_stats()
                        .staves_with_errors_as_slice(),
                ));
            }
            // If no filtering, the HBFs seen is from the total RDHs
            report.add_stat(StatSummary::new(
                "Total HBFs".to_string(),
                self.stats_collector.rdh_stats().hbfs_seen().to_string(),
                None,
            ));

            // If no filtering, the payload size seen is from the total RDHs
            report.add_stat(summerize_data_size(
                self.stats_collector.rdh_stats().rdhs_seen(),
                self.stats_collector.rdh_stats().payload_size(),
            ));
        }

        // Add ALPIDE stats (if they are collected)
        if let Some(alpide_stats) = self.stats_collector.take_alpide_stats() {
            add_alpide_stats_to_report(&mut report, alpide_stats);
        }

        // Add detected attributes
        add_detected_attributes_to_report(&mut report, self.stats_collector.rdh_stats());

        self.append_spinner_msg("... completed");
        if self.spinner.is_some() {
            self.spinner.as_mut().unwrap().abandon();
        }
        report.print();
    }

    fn add_global_stats_to_report(&mut self, report: &mut Report) {
        if self.stats_collector.total_errors() == 0 {
            report.add_stat(StatSummary::new(
                "Total Errors".green().to_string(),
                self.stats_collector.total_errors().green().to_string(),
                None,
            ));
        } else {
            report.add_stat(StatSummary::new(
                "Total Errors".red().to_string(),
                self.stats_collector.total_errors().red().to_string(),
                Some(format_error_codes(
                    self.stats_collector.unique_error_codes_as_slice(),
                )),
            ));
        }

        let (trigger_type_raw, trigger_type_str) =
            self.stats_collector.rdh_stats().run_trigger_type();
        report.add_stat(StatSummary {
            statistic: "Run Trigger Type".to_string(),
            value: format!("{trigger_type_raw:#02X}"),
            notes: trigger_type_str.into_string(),
        });
        report.add_stat(StatSummary::new(
            "Total RDHs".to_string(),
            self.stats_collector.rdh_stats().rdhs_seen().to_string(),
            None,
        ));
        self.stats_collector.finalize();
        report.add_stat(StatSummary::new(
            "Links observed".to_string(),
            format_links_observed(self.stats_collector.rdh_stats().links_as_slice()),
            None,
        ));
        report.add_stat(StatSummary::new(
            "FEE IDs seen".to_string(),
            format_fee_ids(self.stats_collector.rdh_stats().fee_ids_as_slice()),
            None,
        ));
    }

    /// Helper function that builds a vector of the stats associated with the filtered data
    fn add_filtered_stats(&mut self) -> Vec<StatSummary> {
        let mut filtered_stats: Vec<StatSummary> = Vec::new();
        filtered_stats.push(StatSummary::new(
            "RDHs".to_string(),
            self.stats_collector.rdh_stats().rdhs_filtered().to_string(),
            None,
        ));
        // If filtering, the HBFs seen is from the filtered RDHs
        filtered_stats.push(StatSummary::new(
            "HBFs".to_string(),
            self.stats_collector.rdh_stats().hbfs_seen().to_string(),
            None,
        ));

        filtered_stats.push(summerize_data_size(
            self.stats_collector.rdh_stats().rdhs_filtered(),
            self.stats_collector.rdh_stats().payload_size(),
        ));

        if let Some(filter_target) = self.config.filter_target() {
            let filtered_target = match filter_target {
                FilterTarget::Link(link_id) => summerize_filtered_links(
                    link_id,
                    self.stats_collector.rdh_stats().links_as_slice(),
                ),
                FilterTarget::Fee(fee_id) => summerize_filtered_fee_ids(
                    fee_id,
                    self.stats_collector.rdh_stats().fee_ids_as_slice(),
                ),
                FilterTarget::ItsLayerStave(fee_id_no_link) => summerize_filtered_its_layer_staves(
                    fee_id_no_link,
                    self.stats_collector.rdh_stats().layer_staves_as_slice(),
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
            if matches!(
                self.stats_collector.rdh_stats().system_id(),
                Some(SystemId::ITS)
            ) {
                // If no filtering, the layers and staves seen is from the total RDHs
                filtered_stats.push(summerize_layers_staves_seen(
                    self.stats_collector.rdh_stats().layer_staves_as_slice(),
                    self.stats_collector
                        .error_stats()
                        .staves_with_errors_as_slice(),
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
        } else {
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

fn add_alpide_stats_to_report(report: &mut Report, alpide_stats: AlpideStats) {
    let mut alpide_stat: Vec<StatSummary> = Vec::new();

    let readout_flags = alpide_stats.readout_flags();

    alpide_stat.push(StatSummary::new(
        "Chip Trailers seen".to_string(),
        readout_flags.chip_trailers_seen().to_string(),
        None,
    ));

    alpide_stat.push(StatSummary::new(
        "Busy Violations".to_string(),
        readout_flags.busy_violations().to_string(),
        None,
    ));

    alpide_stat.push(StatSummary::new(
        "Data Overrun".to_string(),
        readout_flags.data_overrun().to_string(),
        None,
    ));

    alpide_stat.push(StatSummary::new(
        "Transmission in Fatal".to_string(),
        readout_flags.transmission_in_fatal().to_string(),
        None,
    ));

    alpide_stat.push(StatSummary::new(
        "Flushed Incomplete".to_string(),
        readout_flags.flushed_incomplete().to_string(),
        None,
    ));
    alpide_stat.push(StatSummary::new(
        "Strobe Extended".to_string(),
        readout_flags.strobe_extended().to_string(),
        None,
    ));
    alpide_stat.push(StatSummary::new(
        "Busy Transitions".to_string(),
        readout_flags.busy_transitions().to_string(),
        None,
    ));

    report.add_alpide_stats(tabled::Table::new(alpide_stat));
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
