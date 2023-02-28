use crate::RDH;

use super::{config::Opt, file_pos_tracker::FilePosTracker};

pub trait ScanCDP {
    fn load_rdh_cru<T: RDH>(&mut self) -> Result<T, std::io::Error>;

    fn load_payload_raw(&mut self, payload_size: usize) -> Result<Vec<u8>, std::io::Error>;

    fn load_cdp<T: RDH>(&mut self) -> Result<(T, Vec<u8>), std::io::Error> {
        let rdh: T = self.load_rdh_cru()?;
        let payload = self.load_payload_raw(rdh.get_payload_size() as usize)?;
        Ok((rdh, payload))
    }
}
/// Allows reading an RDH from a file
/// Optionally, the RDH can be filtered by link ID
/// # Example
pub struct FileScanner {
    pub reader: std::io::BufReader<std::fs::File>,
    pub tracker: FilePosTracker,
    pub stats_sender_ch: std::sync::mpsc::Sender<super::stats::StatType>,
    pub link_to_filter: Option<u8>,
    unique_links_observed: Vec<u8>,
}

impl FileScanner {
    pub fn new(
        config: std::sync::Arc<Opt>,
        reader: std::io::BufReader<std::fs::File>,
        tracker: FilePosTracker,
        stats_sender_ch: std::sync::mpsc::Sender<super::stats::StatType>,
    ) -> Self {
        FileScanner {
            reader,
            tracker,
            stats_sender_ch,
            link_to_filter: config.filter_link(),
            unique_links_observed: vec![],
        }
    }
    pub fn default(
        config: &Opt,
        stats_sender_ch: std::sync::mpsc::Sender<super::stats::StatType>,
    ) -> Self {
        let reader = crate::setup_buffered_reading(&config);
        FileScanner {
            reader,
            stats_sender_ch,
            tracker: FilePosTracker::new(),
            link_to_filter: config.filter_link(),
            unique_links_observed: vec![],
        }
    }
}

impl ScanCDP for FileScanner {
    /// Reads the next RDH from file
    /// If a link filter is set, it checks if the RDH matches the chosen link and returns it if it does.
    /// If it doesn't match, it jumps to the next RDH and tries again.
    /// If no link filter is set, it simply returns the RDH.
    fn load_rdh_cru<T: RDH>(&mut self) -> Result<T, std::io::Error> {
        let rdh: T = RDH::load(&mut self.reader)?;
        let current_link_id = rdh.get_link_id();
        self.stats_sender_ch
            .send(super::stats::StatType::RDHsSeen(1))
            .unwrap();
        if self.unique_links_observed.contains(&current_link_id) == false {
            self.unique_links_observed.push(current_link_id);
            self.stats_sender_ch
                .send(super::stats::StatType::LinksObserved(current_link_id))
                .unwrap();
        }

        match self.link_to_filter {
            // Matches if a link is set and it is the same as the current RDH
            Some(x) if x == current_link_id => {
                self.stats_sender_ch
                    .send(super::stats::StatType::RDHsFiltered(1))
                    .unwrap();
                // no jump. current pos -> start of payload
                return Ok(rdh);
            }
            // Matches if no link is set
            None => {
                // No jump, current pos -> start of payload
                return Ok(rdh);
            }
            // Matches all remaining cases (link set, but not the same as the current RDH)
            _ => {
                // Set tracker to jump to next RDH and try again
                self.reader
                    .seek_relative(self.tracker.next(rdh.get_offset_to_next() as u64))?;

                return self.load_rdh_cru();
            }
        }
    }

    /// Reads the next payload from file, using the payload size from the RDH
    fn load_payload_raw(&mut self, payload_size: usize) -> Result<Vec<u8>, std::io::Error> {
        let mut payload = vec![0; payload_size];
        std::io::Read::read_exact(&mut self.reader, &mut payload)?;
        debug_assert!(payload.len() == payload_size);
        self.stats_sender_ch
            .send(super::stats::StatType::PayloadSize(payload_size as u32))
            .unwrap();
        Ok(payload)
    }
    /// Reads the next CDP from file
    fn load_cdp<T: RDH>(&mut self) -> Result<(T, Vec<u8>), std::io::Error> {
        let rdh: T = self.load_rdh_cru()?;
        let payload = self.load_payload_raw(rdh.get_payload_size() as usize)?;
        Ok((rdh, payload))
    }
}
#[cfg(test)]
mod tests {

    use crate::{
        util::stats::{self, Stats},
        words::rdh::RdhCRUv7,
    };

    use super::*;
    #[test]
    #[ignore] // Large test ignored in normal cases, useful for debugging
    fn full_file_filter() {
        let config: Opt = <Opt as structopt::StructOpt>::from_iter(&[
            "fastpasta",
            "-s",
            "-f",
            "0",
            "../fastpasta_test_files/data_ols_ul.raw",
            "-o test_filter_link.raw",
        ]);
        println!("{:#?}", config);
        let (send_stats_channel, recv_stats_channel): (
            std::sync::mpsc::Sender<stats::StatType>,
            std::sync::mpsc::Receiver<stats::StatType>,
        ) = std::sync::mpsc::channel();

        let thread_stopper = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

        let mut stats = Stats::new(&config, recv_stats_channel, thread_stopper.clone());
        let stats_thread = std::thread::spawn(move || {
            stats.run();
        });

        let mut scanner = FileScanner::default(&config, send_stats_channel);

        let mut link0_rdh_data: Vec<RdhCRUv7> = vec![];
        let mut link0_payload_data: Vec<Vec<u8>> = vec![];

        let tmp_rdh: RdhCRUv7 = scanner.load_rdh_cru().unwrap();
        link0_payload_data.push(
            scanner
                .load_payload_raw(tmp_rdh.get_payload_size() as usize)
                .unwrap(),
        );
        link0_rdh_data.push(tmp_rdh);

        assert!(link0_rdh_data.len() == 1);
        assert!(link0_payload_data.len() == 1);
        assert!(link0_payload_data.first().unwrap().len() > 1);

        let rdh_validator = crate::validators::rdh::RDH_CRU_V7_VALIDATOR;

        let tmp_rdh = link0_rdh_data.first().unwrap();
        println!("RDH: {}", tmp_rdh);
        match rdh_validator.sanity_check(&link0_rdh_data.first().unwrap()) {
            Ok(_) => (),
            Err(e) => {
                println!("Sanity check failed: {}", e);
            }
        }

        let mut loop_count = 0;
        while let Ok(rdh) = scanner.load_rdh_cru::<RdhCRUv7>() {
            println!("{rdh}");
            loop_count += 1;
            print!("{} ", loop_count);
            link0_payload_data.push(
                scanner
                    .load_payload_raw(rdh.get_payload_size() as usize)
                    .unwrap(),
            );
            link0_rdh_data.push(rdh);

            match rdh_validator.sanity_check(&link0_rdh_data.last().unwrap()) {
                Ok(_) => (),
                Err(e) => {
                    println!("Sanity check failed: {}", e);
                }
            }
        }

        println!(
            "Total RDHs: {}, Payloads: {}",
            link0_rdh_data.len(),
            link0_payload_data.len()
        );

        link0_rdh_data.iter().for_each(|rdh| {
            println!("{rdh}");
        });

        stats_thread.join().unwrap();
    }
}
