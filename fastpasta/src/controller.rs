//! Contains the [Controller] that collects stats and reports errors.
//! It also controls the stop flag, which can be used to stop the program if a fatal error occurs, or if the config contains a max number of errors to tolerate.
//! Finally when the event loop breaks (at the end of execution), it will print a summary of the stats collected, using the Report struct.
//!
//! Also contains the convenience [init_controller] function, which spawns a thread with the [Controller] running, and returns the thread handle, the channel to send stats to, and the stop flag.

use super::*;
use crate::stats::err_printer::ErrPrinter;
use std::io::Write;

/// Spawns a thread with the [Controller] running, and returns the thread handle, the channel to send stats to, and the stop flag.
pub fn init_controller<C: Config + 'static>(
    config: &'static C,
) -> (
    JoinHandle<()>,
    flume::Sender<StatType>,
    Arc<AtomicBool>,
    Arc<AtomicBool>,
) {
    log::trace!("Initializing stats controller");
    let mut stats = Controller::new(config);
    let stats_send_chan = stats.send_channel();
    let thread_stop_flag = stats.end_processing_flag();
    let any_errors_flag = stats.any_errors_flag();

    let stats_thread = Builder::new()
        .name("stats_thread".to_string())
        .spawn(move || {
            stats.run();
        })
        .expect("Failed to spawn stats thread");
    (
        stats_thread,
        stats_send_chan,
        thread_stop_flag,
        any_errors_flag,
    )
}

/// The Controller receives stats and builds a summary report that is printed at the end of execution.
pub struct Controller<C: Config + 'static> {
    stats_collector: StatsCollector,
    /// Time from [Controller] is instantiated, to all data processing threads disconnected their [StatType] producer channel.
    pub processing_time: Instant,
    config: &'static C,
    max_tolerate_errors: u32,
    // The channel where stats are received from other threads.
    stats_recv_chan: flume::Receiver<StatType>,
    // The channel stats are sent through, stored so that a clone of the channel can be returned easily
    // Has to be an option so that it can be set to None when the event loop starts.
    // Once run is called no producers that don't already have a channel to send stats through, will be able to get one.
    // This is because the event loop breaks when all sender channels are dropped, and if the Controller keeps a reference to the channel, it will cause a deadlock.
    stats_send_chan: Option<flume::Sender<StatType>>,
    end_processing_flag: Arc<AtomicBool>,
    any_errors_flag: Arc<AtomicBool>,
    spinner: Option<ProgressBar>,
    spinner_message: String,
}
impl<C: Config + 'static> Controller<C> {
    /// Creates a new [Controller] from a [Config], a [flume::Receiver] for [StatType], and a [Arc] of an [AtomicBool] that is used to signal to other threads to exit if a fatal error occurs.
    pub fn new(global_config: &'static C) -> Self {
        let (stats_send_chan, stats_recv_chan): (
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
            processing_time: Instant::now(),
            max_tolerate_errors: global_config.max_tolerate_errors(),
            stats_recv_chan,
            stats_send_chan: Some(stats_send_chan),
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
        if self.stats_send_chan.is_none() {
            log::error!("Controller send channel is none, most likely it is already running and does not accept new producers");
            panic!("Controller send channel is none, most likely it is already running and does not accept new producers");
        }
        self.stats_send_chan.as_ref().unwrap().clone()
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
        self.stats_send_chan = None;

        // While loop breaks when an error is received from the channel, which means the channel is disconnected
        while let Ok(stats_update) = self.stats_recv_chan.recv() {
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
            self.process_stats();

            // Print the summary report if any RDHs were seen. If not, it's likely that an early error occurred and no data was processed.
            if self.stats_collector.any_rdhs_seen() {
                // New spinner/progress bar
                self.new_spinner_with_prefix("Generating report".to_string());
                self.print();
            }
        }
        if self.stats_collector.any_errors() {
            self.any_errors_flag.store(true, Ordering::SeqCst);
        }

        // Stats collector will serialize and write out stats if the config specifies it
        if self.config.stats_output_mode() != DataOutputMode::None {
            self.stats_collector.write_stats(
                &self.config.stats_output_mode(),
                self.config.stats_output_format().unwrap(),
            );
        }

        // User supplied a stats file to compare against, validate the match
        if let Some(input_stats) = self.config.input_stats_file() {
            log::info!("Validating input stats file against collected stats");
            let input_stats_str =
                fs::read_to_string(input_stats).expect("Failed to read input stats file");

            let input_stats_collector: StatsCollector = if input_stats.extension().unwrap()
                == "json"
            {
                serde_json::from_str(&input_stats_str)
                    .expect("Failed to deserialize input stats file")
            } else if input_stats.extension().unwrap() == "toml" {
                toml::from_str(&input_stats_str).expect("Failed to deserialize input stats file")
            } else {
                // Should've already been validated when parsing the command-line arguments
                panic!("Invalid input stats file extension, must be .json or .toml")
            };

            if self
                .stats_collector
                .validate_other_stats(&input_stats_collector, self.config.mute_errors())
                .is_err()
            {
                self.any_errors_flag.store(true, Ordering::SeqCst);
                log::warn!("Input stats did not match collected stats");
            } else {
                log::info!("Input stats matched collected stats");
            }
        }
    }

    fn update(&mut self, stat: StatType) {
        match stat {
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
            StatType::HBFsSeen(_) => {
                self.stats_collector.collect(stat);
                if self.spinner.is_some() {
                    self.spinner.as_mut().unwrap().set_prefix(format!(
                        "Analyzing {hbfs} HBFs",
                        hbfs = self.stats_collector.hbfs_seen()
                    ))
                };
            }
            StatType::RunTriggerType((raw_tt, tt_str)) => {
                log::debug!("Run trigger type determined to be {raw_tt:#0x}: {tt_str}");
                self.stats_collector
                    .collect(StatType::RunTriggerType((raw_tt, tt_str)));
            }
            StatType::Error(msg) => {
                // Stop processing any error messages
                if self.stats_collector.any_fatal_err() {
                    log::trace!("Fatal error already seen, ignoring error: {msg}");
                    return;
                }

                self.stats_collector.collect(StatType::Error(msg));

                self.set_spinner_msg(
                    format!(
                        "{err_cnt} Errors in data!",
                        err_cnt = self.stats_collector.err_count()
                    )
                    .red()
                    .to_string(),
                );

                if self.max_tolerate_errors > 0 {
                    log::trace!("Error count: {}", self.stats_collector.err_count());
                    if self.stats_collector.err_count() == self.max_tolerate_errors as u64 {
                        log::trace!("Errors reached maximum tolerated errors, exiting...");
                        self.end_processing_flag.store(true, Ordering::SeqCst);
                    }
                }
            }
            StatType::Fatal(err) => {
                // Stop processing any error messages
                if self.stats_collector.any_fatal_err() {
                    log::trace!("Fatal error already seen, ignoring error: {err}");
                    return;
                }
                self.end_processing_flag.store(true, Ordering::SeqCst);
                log::error!("FATAL: {err}\nShutting down...");
                self.stats_collector.collect(StatType::Fatal(err));
            }
        }
    }

    fn process_stats(&mut self) {
        // New spinner/progress bar if there's any errors
        if self.stats_collector.err_count() > 0 {
            self.new_spinner_with_prefix(
                format!(
                    "Processing {err_count} error messages",
                    err_count = self.stats_collector.err_count()
                )
                .yellow()
                .to_string(),
            );
            self.stats_collector.finalize(self.config.mute_errors());
            self.spinner.as_mut().unwrap().abandon();
        } else {
            self.stats_collector.finalize(self.config.mute_errors());
        }

        if self.stats_collector.any_errors() && !self.config.mute_errors() {
            // Print the errors, limited if there's a max error limit set
            ErrPrinter::new(
                if self.config.max_tolerate_errors() > 0 {
                    Some(self.config.max_tolerate_errors())
                } else {
                    None
                },
                self.config.error_code_filter(),
            )
            .print(
                self.stats_collector.error_stats().errors_as_slice_iter(),
                self.stats_collector.unique_error_codes_as_slice(),
            );
        }
    }

    /// Builds and prints the report
    fn print(&mut self) {
        let mut report = stats::stats_report::make_report(
            self.processing_time.elapsed(),
            &mut self.stats_collector,
            self.config.filter_target(),
        );
        self.append_spinner_msg("... completed");
        if self.spinner.is_some() {
            self.spinner.as_mut().unwrap().abandon();
        }

        let mut lock = io::stdout().lock();
        if let Err(e) = writeln!(lock, "{}", report.format()) {
            if e.kind() == io::ErrorKind::BrokenPipe {
                log::warn!("Broken pipe, stdout was closed before report could be written");
            } else {
                log::error!("Failed to write report to stdout: {e}");
            }
        }
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
    pb.enable_steady_tick(Duration::from_millis(120));
    pb
}

#[cfg(test)]
mod tests {
    use super::*;

    static CONFIG_TEST_INIT_CONTROLLER: OnceLock<MockConfig> = OnceLock::new();

    #[test]
    fn test_init_controller() {
        let mock_config = MockConfig::default();
        CONFIG_TEST_INIT_CONTROLLER.set(mock_config).unwrap();

        let (handle, send_ch, stop_flag, _errors_flag) =
            init_controller(CONFIG_TEST_INIT_CONTROLLER.get().unwrap());

        // Stop flag should be false
        assert!(!stop_flag.load(Ordering::SeqCst));

        // Send RDH version seen
        send_ch.send(StatType::RdhVersion(7)).unwrap();

        // Send Data format seen
        send_ch.send(StatType::DataFormat(99)).unwrap();

        // Send Run Trigger Type
        send_ch
            .send(StatType::RunTriggerType((0xBEEF, "BEEF".to_owned().into())))
            .unwrap();

        // Send rdh seen stat
        send_ch.send(StatType::RDHSeen(1)).unwrap();

        // Send a fatal error that should cause the stop flag to be set
        send_ch
            .send(StatType::Fatal("Test fatal error".to_string().into()))
            .unwrap();

        // Stop the controller by dropping the sender channel
        drop(send_ch);

        // Wait for the controller to stop
        handle.join().unwrap();

        // Stop flag should be true
        assert!(stop_flag.load(Ordering::SeqCst));
    }
}
