use crate::RDH;

use super::{config::Opt, file_pos_tracker::FilePosTracker, stats::Stats};

pub trait ScanCDP {
    fn load_rdh_cru<T: RDH>(&mut self) -> Result<T, std::io::Error>;

    fn load_payload(&mut self) -> Result<Vec<u8>, std::io::Error>;

    fn load_cdp<T: RDH>(&mut self) -> Result<(T, Vec<u8>), std::io::Error> {
        let rdh = self.load_rdh_cru()?;
        let payload = self.load_payload()?;
        Ok((rdh, payload))
    }
}

pub struct FileScanner<'a> {
    pub reader: std::io::BufReader<std::fs::File>,
    pub tracker: FilePosTracker,
    pub stats: &'a mut Stats,
    pub link_to_filter: Option<u8>,
}

impl<'a> FileScanner<'a> {
    pub fn new(
        reader: std::io::BufReader<std::fs::File>,
        tracker: FilePosTracker,
        stats: &'a mut Stats,
        config: &'a Opt,
    ) -> Self {
        FileScanner {
            reader,
            tracker,
            stats,
            link_to_filter: config.filter_link(),
        }
    }
}

impl ScanCDP for FileScanner<'_> {
    fn load_rdh_cru<T: RDH>(&mut self) -> Result<T, std::io::Error> {
        let rdh: T = RDH::load(&mut self.reader)?;
        self.stats.rdhs_seen += 1;
        self.tracker
            .update_next_payload_size(rdh.get_payload_size() as usize);

        match self.link_to_filter {
            Some(x) if x == rdh.get_link_id() => {
                self.stats.rdhs_filtered += 1;
                // no jump. current pos -> start of payload
                return Ok(rdh);
            }
            None => {
                // No jump, current pos -> start of payload
                return Ok(rdh);
            }
            _ => {
                // Set tracker to jump to next RDH and try again
                self.reader
                    .seek_relative(self.tracker.next(rdh.get_payload_size() as u64))?;

                return self.load_rdh_cru();
            }
        }
    }

    fn load_payload(&mut self) -> Result<Vec<u8>, std::io::Error> {
        let payload_size = self.tracker.next_payload_size();
        debug_assert!(payload_size > 0);
        let mut payload = vec![0; payload_size];
        std::io::Read::read_exact(&mut self.reader, &mut payload)?;
        debug_assert!(payload.len() == payload_size);
        self.stats.payload_size += payload_size as u64;
        Ok(payload)
    }
    fn load_cdp<T: RDH>(&mut self) -> Result<(T, Vec<u8>), std::io::Error> {
        let rdh = self.load_rdh_cru()?;
        let payload = self.load_payload()?;
        Ok((rdh, payload))
    }
}
#[cfg(test)]
mod tests {
    use crate::data_words::rdh::RdhCRUv7;

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

        let file_tracker = FilePosTracker::new();
        let mut stats = Stats::new();
        let reader = crate::setup_buffered_reading(&config);

        let mut scanner = FileScanner::new(reader, file_tracker, &mut stats, &config);

        let mut link0_rdh_data: Vec<RdhCRUv7> = vec![];
        let mut link0_payload_data: Vec<Vec<u8>> = vec![];

        link0_rdh_data.push(scanner.load_rdh_cru().unwrap());
        link0_payload_data.push(scanner.load_payload().unwrap());
        assert!(link0_rdh_data.len() == 1);
        assert!(link0_payload_data.len() == 1);
        assert!(link0_payload_data.first().unwrap().len() > 1);

        let rdh_validator = crate::validators::rdh::RDH_CRU_V7_VALIDATOR;

        RdhCRUv7::print_header_text();
        link0_rdh_data.first().unwrap().print();
        match rdh_validator.sanity_check(&link0_rdh_data.first().unwrap()) {
            Ok(_) => (),
            Err(e) => {
                println!("Sanity check failed: {}", e);
            }
        }

        while let Ok(rdh) = scanner.load_rdh_cru() {
            link0_rdh_data.push(rdh);
            link0_payload_data.push(scanner.load_payload().unwrap());
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

        RdhCRUv7::print_header_text();
        link0_rdh_data.iter().enumerate().for_each(|(i, rdh)| {
            rdh.print();
            if i % 30 == 0 {
                RdhCRUv7::print_header_text();
            }
        });
    }
}
