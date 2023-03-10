use std::sync::{
    atomic::{AtomicBool, AtomicU32},
    Arc,
};

use log::{error, info};

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
        if self.max_tolerate_errors == 0 {
            eprintln!("Total errors: {}", self.non_atomic_total_errors);
        } else {
            eprintln!(
                "Total errors: {}",
                self.total_errors.load(std::sync::atomic::Ordering::SeqCst)
            );
        }
        eprintln!("Total RDHs: {}", self.rdhs_seen);
        eprintln!("Filtered:");

        let mut filter_links_res: String = self
            .links_to_filter
            .iter()
            .filter(|x| self.links_observed.contains(x))
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        if filter_links_res.is_empty() {
            filter_links_res = "<<all-links>>".to_string();
        }
        let not_filtered = self
            .links_to_filter
            .iter()
            .filter(|x| !self.links_observed.contains(x))
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        if !not_filtered.is_empty() {
            filter_links_res.push_str(&format!(" (not found: {not_filtered})"));
        }
        eprintln!("   {:<3}{:>6}", "Links selected: ", filter_links_res);
        eprintln!("   {:<3}{:>7}", "RDHs: ", self.rdhs_filtered);
        eprint!("   {:<3}", "Payload ");
        match self.payload_size {
            0..=1024 => eprintln!("{} B", self.payload_size),
            1025..=1048576 => {
                eprintln!("{:.3} KB", self.payload_size as f64 / 1024_f64)
            }
            1048577..=1073741824 => {
                eprintln!("{:.3} MB", self.payload_size as f64 / 1048576_f64)
            }
            _ => eprintln!("{:.3} GB", self.payload_size as f64 / 1073741824_f64),
        }
        let mut observed_links = self.links_observed.clone();
        observed_links.sort();
        eprintln!("Links observed during scan: {observed_links:?}");
        eprintln!("Processing time: {:?}", self.processing_time.elapsed());
    }
    pub fn print_time(&self) {
        eprintln!("Processing time: {:?}", self.processing_time.elapsed());
    }
}
