use crate::stats::report::{Report, StatSummary};
use log::{error, info};
use std::sync::{
    atomic::{AtomicBool, AtomicU32},
    Arc,
};

use super::config::Opt;
pub enum StatType {
    Error(String),
    RDHsSeen(u8),
    RDHsFiltered(u8),
    PayloadSize(u32),
    LinksObserved(u8),
    ProcessingTime,
}

pub struct Stats {
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
    links_to_filter: Vec<u8>,
}
impl Stats {
    pub fn new(
        config: &Opt,
        recv_stats_channel: std::sync::mpsc::Receiver<StatType>,
        end_processing_flag: Arc<AtomicBool>,
    ) -> Self {
        Stats {
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
            links_to_filter: if let Some(links) = config.filter_link() {
                links
            } else {
                Vec::new()
            },
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
            StatType::PayloadSize(val) => self.payload_size += val as u64,
            StatType::LinksObserved(val) => self.links_observed.push(val),
            StatType::ProcessingTime => info!("{:?}", self.processing_time.elapsed()),
        }
    }

    pub fn print(&self) {
        let mut report = Report::new(self.processing_time.elapsed());
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
        // Filtered stats
        let mut filtered_stats: Vec<StatSummary> = Vec::new();
        filtered_stats.push(StatSummary::new(
            "RDHs".to_string(),
            self.rdhs_filtered.to_string(),
            None,
        ));
        let filtered_links =
            summerize_filtered_links(&self.links_to_filter, self.links_observed.clone());
        filtered_stats.push(filtered_links);
        report.add_filter_stats(tabled::Table::new(filtered_stats));

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
        report.add_stat(StatSummary::new(
            "Payload Size".to_string(),
            payload_string,
            None,
        ));
        report.print();
    }
    pub fn print_time(&self) {
        eprintln!("Processing time: {:?}", self.processing_time.elapsed());
    }
}

/// Helper functions to format the summary
fn summerize_filtered_links(links_to_filter: &Vec<u8>, links_observed: Vec<u8>) -> StatSummary {
    let mut filtered_links_stat = StatSummary::new("Link IDs".to_string(), "".to_string(), None);
    // Format links that were filtered, separated by commas
    let filter_links_res: String = links_to_filter
        .iter()
        .filter(|x| links_observed.contains(x))
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(", ");

    if filter_links_res.is_empty() {
        // If no links specified by the user, all links were filtered
        if links_to_filter.is_empty() {
            filtered_links_stat.value = "<<all-links>>".to_string();
        }
        // If links were specified and none of those links were found, no links were filtered
        else {
            filtered_links_stat.value = "<<none>>".to_string();
        }
    } else {
        filtered_links_stat.value = filter_links_res;
    }

    // Format links that were specified but not found
    let not_filtered = links_to_filter
        .iter()
        .filter(|x| !links_observed.contains(x))
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(", ");

    if !not_filtered.is_empty() {
        let not_filtered_str = format!(" (not found: {not_filtered})");
        filtered_links_stat.notes = not_filtered_str;
    }
    filtered_links_stat
}
