use crate::{
    data_words::rdh::{RdhCRUv6, RdhCRUv7},
    GbtWord,
};

use super::{config::Opt, file_pos_tracker::FilePosTracker, stats::Stats};

pub trait LoadRdhCru<T> {
    fn load_rdh_cru(&mut self) -> Result<T, std::io::Error>
    where
        T: GbtWord;
}

pub trait LoadPayload<T, U> {
    fn load_payload(&mut self) -> Result<Vec<U>, std::io::Error>;
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

impl LoadRdhCru<RdhCRUv7> for FileScanner<'_> {
    fn load_rdh_cru(&mut self) -> Result<RdhCRUv7, std::io::Error> {
        let rdh = RdhCRUv7::load(&mut self.reader)?;
        self.stats.rdhs_seen += 1;
        self.tracker.update_next_payload_size(&rdh);

        match self.link_to_filter {
            Some(x) if x == rdh.link_id => {
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
                    .seek_relative(self.tracker.next(rdh.offset_new_packet as u64))?;

                return self.load_rdh_cru();
            }
        }
    }
}

impl LoadRdhCru<RdhCRUv6> for FileScanner<'_> {
    fn load_rdh_cru(&mut self) -> Result<RdhCRUv6, std::io::Error> {
        let rdh = RdhCRUv6::load(&mut self.reader)?;
        self.tracker.next(rdh.offset_new_packet as u64);
        self.stats.rdhs_seen += 1;

        //        if rdh.link_id != self.

        Ok(rdh)
    }
}

impl LoadPayload<RdhCRUv7, u8> for FileScanner<'_> {
    fn load_payload(&mut self) -> Result<Vec<u8>, std::io::Error> {
        let payload_size = self.tracker.next_payload_size();
        let mut payload_buf: Vec<u8> = vec![0; payload_size];
        std::io::Read::read_exact(&mut self.reader, &mut payload_buf)?;
        debug_assert!(payload_buf.len() == payload_size);
        Ok(payload_buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_filter_link() {
        let config: Opt = <Opt as structopt::StructOpt>::from_iter(&[
            "fastpasta",
            "-d",
            "-f",
            "0",
            "../fastpasta_test_files/data_ols_ul.raw",
            "-o test_filter_link.raw",
        ]);
        println!("{:#?}", config);

        // OLD TEST WITH FILTERLINK STRUCT

        // let mut filter_link = FilterLink::new(&config, 1024);

        // assert_eq!(filter_link.link_to_filter, 0);
        // assert_eq!(filter_link.filtered_rdhs_buffer.len(), 0);
        // assert_eq!(filter_link.filtered_payload_buffers.len(), 0);

        // let file = crate::file_open_read_only(&config.file()).unwrap();
        // let mut buf_reader = crate::buf_reader_with_capacity(file, 1024 * 10);
        // let file_tracker = FilePosTracker::new();
        // let rdh = RdhCRUv7::load(&mut buf_reader).unwrap();
        // RdhCRUv7::print_header_text();
        // rdh.print();
        // assert!(filter_link.filter_link(&mut buf_reader, rdh));
        // // This function currently corresponds to the unlink function on Unix and the DeleteFile function on Windows. Note that, this may change in the future.
        // // More info: https://doc.rust-lang.org/std/fs/fn.remove_file.html
        // std::fs::remove_file(Opt::output(&config).as_ref().unwrap()).unwrap();

        // filter_link
        //     .filtered_payload_buffers
        //     .iter()
        //     .for_each(|payload| {
        //         println!("Payload size: {}", payload.len());
        //     });

        // let rdh2 = RdhCRUv7::load(&mut buf_reader).unwrap();
        // rdh2.print();
    }

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
