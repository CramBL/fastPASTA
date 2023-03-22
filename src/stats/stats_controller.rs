use crate::{
    stats::report::{Report, StatSummary},
    util::lib::Config,
};
use log::{error, info};
use std::sync::{
    atomic::{AtomicBool, AtomicU32},
    Arc,
};

pub enum StatType {
    Fatal(String), // Fatal error, stop processing
    Error(String),
    RDHsSeen(u8),
    RDHsFiltered(u8),
    PayloadSize(u32),
    LinksObserved(u8),
    ProcessingTime,
    RdhVersion(u8),
    // TimeFrameLength(u32),
    DataFormat(u8),
    HBFsSeen(u32),
    LayerStaveSeen { layer: u8, stave: u8 },
}

pub struct StatsController {
    pub rdhs_seen: u64,
    pub rdhs_filtered: u64,
    pub payload_size: u64,
    pub links_observed: Vec<u8>,
    pub processing_time: std::time::Instant,
    total_errors: AtomicU32,
    non_atomic_total_errors: u64,
    max_tolerate_errors: u32,
    recv_stats_channel: std::sync::mpsc::Receiver<StatType>,
    end_processing_flag: Arc<AtomicBool>,
    link_to_filter: Option<u8>,
    rdh_version: u8,
    data_formats_observed: Vec<u8>,
    hbfs_seen: u32,
    fatal_error: Option<String>,
    layers_staves_seen: Vec<(u8, u8)>,
}
impl StatsController {
    pub fn new(
        config: &impl Config,
        recv_stats_channel: std::sync::mpsc::Receiver<StatType>,
        end_processing_flag: Arc<AtomicBool>,
    ) -> Self {
        StatsController {
            rdhs_seen: 0,
            rdhs_filtered: 0,
            payload_size: 0,
            links_observed: Vec::new(),
            processing_time: std::time::Instant::now(),
            total_errors: AtomicU32::new(0),
            max_tolerate_errors: config.max_tolerate_errors(),
            non_atomic_total_errors: 0,
            recv_stats_channel,
            end_processing_flag,
            link_to_filter: config.filter_link(),
            rdh_version: 0,
            data_formats_observed: Vec::new(),
            hbfs_seen: 0,
            fatal_error: None,
            layers_staves_seen: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.recv_stats_channel.recv() {
                Ok(stats_update) => self.update(stats_update),
                Err(_) => {
                    self.print();
                    break;
                }
            }
        }
    }

    pub fn update(&mut self, stat: StatType) {
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
            StatType::ProcessingTime => info!("{:?}", self.processing_time.elapsed()),
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
        }
    }

    pub fn print(&self) {
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
        report.add_stat(StatSummary::new(
            "Total RDHs".to_string(),
            self.rdhs_seen.to_string(),
            None,
        ));
        // Sort and format links observed
        let mut observed_links = self.links_observed.clone();
        observed_links.sort();
        let observed_links_string = observed_links
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        report.add_stat(StatSummary::new(
            "Links observed during scan".to_string(),
            observed_links_string,
            None,
        ));
        // Sort and format layers and staves seen
        let mut layers_staves_seen = self.layers_staves_seen.clone();
        layers_staves_seen.sort();
        let layers_staves_seen_string = layers_staves_seen
            .iter()
            .map(|(layer, stave)| format!("L{layer}_{stave}"))
            .collect::<Vec<String>>()
            .join(", ");
        // Format and add payload size seen/loaded
        let payload_string = match self.payload_size {
            0..=1024 => format!("{} B", self.payload_size),
            1025..=1048576 => {
                format!("{:.3} KiB", self.payload_size as f64 / 1024_f64)
            }
            1048577..=1073741824 => {
                format!("{:.3} MiB", self.payload_size as f64 / 1048576_f64)
            }
            _ => format!("{:.3} GiB", self.payload_size as f64 / 1073741824_f64),
        };
        // If no filtering, the HBFs seen is from the total RDHs
        if self.link_to_filter.is_none() {
            report.add_stat(StatSummary::new(
                "Total HBFs".to_string(),
                self.hbfs_seen.to_string(),
                None,
            ));
            // If no filtering, the layers and staves seen is from the total RDHs
            report.add_stat(StatSummary::new(
                "Layers and Staves seen".to_string(),
                layers_staves_seen_string,
                None,
            ));
            // If no filtering, the payload size seen is from the total RDHs
            report.add_stat(StatSummary::new(
                "Total Payload Size".to_string(),
                payload_string,
                None,
            ));
        } else {
            let mut filtered_stats: Vec<StatSummary> = Vec::new();
            filtered_stats.push(StatSummary::new(
                "RDHs".to_string(),
                self.rdhs_filtered.to_string(),
                None,
            ));
            filtered_stats.push(StatSummary::new(
                "HBFs".to_string(),
                self.hbfs_seen.to_string(),
                None,
            ));
            let payload_string = match self.payload_size {
                0..=1024 => format!("{} B", self.payload_size),
                1025..=1048576 => {
                    format!("{:.3} KiB", self.payload_size as f64 / 1024_f64)
                }
                1048577..=1073741824 => {
                    format!("{:.3} MiB", self.payload_size as f64 / 1048576_f64)
                }
                _ => format!("{:.3} GiB", self.payload_size as f64 / 1073741824_f64),
            };
            filtered_stats.push(StatSummary::new(
                "Total Payload Size".to_string(),
                payload_string,
                None,
            ));
            let filtered_links =
                summerize_filtered_links(self.link_to_filter.unwrap(), self.links_observed.clone());
            filtered_stats.push(filtered_links);
            filtered_stats.push(StatSummary::new(
                "Layers and Staves seen".to_string(),
                layers_staves_seen_string,
                None,
            ));
            report.add_filter_stats(tabled::Table::new(filtered_stats));
        }

        // Add detected attributes
        report.add_detected_attribute("RDH Version".to_string(), self.rdh_version.to_string());
        let mut observed_data_formats = self.data_formats_observed.clone();
        if observed_data_formats.len() > 1 {
            observed_data_formats.sort();
            log::error!(
                "Multiple data formats observed: {:?}",
                observed_data_formats
            );
        }
        let observed_data_formats_string = observed_data_formats
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        report.add_detected_attribute("Data Format".to_string(), observed_data_formats_string);

        report.print();
    }
    pub fn print_time(&self) {
        eprintln!("Processing time: {:?}", self.processing_time.elapsed());
    }
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
