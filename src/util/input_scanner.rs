use super::stats_controller::StatType;
use super::{
    bufreader_wrapper::BufferedReaderWrapper, config::Opt, mem_pos_tracker::MemPosTracker,
};
use crate::words::rdh::{Rdh0, RDH};
use std::io::Read;

/// Trait for a scanner that reads CDPs from a file or stdin
pub trait ScanCDP {
    fn load_rdh_cru<T: RDH>(&mut self) -> Result<T, std::io::Error>;

    fn load_payload_raw(&mut self, payload_size: usize) -> Result<Vec<u8>, std::io::Error>;

    fn load_cdp<T: RDH>(&mut self) -> Result<(T, Vec<u8>), std::io::Error> {
        let rdh: T = self.load_rdh_cru()?;
        let payload = self.load_payload_raw(rdh.payload_size() as usize)?;
        Ok((rdh, payload))
    }

    fn load_next_rdh_to_filter<T: RDH>(&mut self) -> Result<T, std::io::Error>;
}

/// Scans data received through a BufferedReaderWrapper, tracks the position in memory and sends stats to the stats controller.
///
/// Uses the config to filter for user specified links.
/// Implements ScanCDP for a BufferedReaderWrapper.
pub struct InputScanner<R: ?Sized + BufferedReaderWrapper> {
    pub reader: Box<R>,
    pub tracker: MemPosTracker,
    pub stats_controller_sender_ch: std::sync::mpsc::Sender<StatType>,
    pub link_to_filter: Option<Vec<u8>>,
    unique_links_observed: Vec<u8>,
    initial_rdh0: Option<Rdh0>,
}

impl<R: ?Sized + BufferedReaderWrapper> InputScanner<R> {
    pub fn new(
        config: std::sync::Arc<Opt>,
        reader: Box<R>,
        tracker: MemPosTracker,
        stats_controller_sender_ch: std::sync::mpsc::Sender<StatType>,
    ) -> Self {
        InputScanner {
            reader,
            tracker,
            stats_controller_sender_ch,
            link_to_filter: config.filter_link(),
            unique_links_observed: vec![],
            initial_rdh0: None,
        }
    }
    pub fn new_from_rdh0(
        config: std::sync::Arc<Opt>,
        reader: Box<R>,
        tracker: MemPosTracker,
        stats_controller_sender_ch: std::sync::mpsc::Sender<StatType>,
        rdh0: Rdh0,
    ) -> Self {
        InputScanner {
            reader,
            tracker,
            stats_controller_sender_ch,
            link_to_filter: config.filter_link(),
            unique_links_observed: vec![],
            initial_rdh0: Some(rdh0),
        }
    }
    fn report_rdh_seen(&self) {
        self.stats_controller_sender_ch
            .send(StatType::RDHsSeen(1))
            .unwrap();
    }
    fn report_link_seen(&self, link_id: u8) {
        self.stats_controller_sender_ch
            .send(StatType::LinksObserved(link_id))
            .unwrap();
    }
    fn report_payload_size(&self, payload_size: usize) {
        self.stats_controller_sender_ch
            .send(StatType::PayloadSize(payload_size as u32))
            .unwrap();
    }
    fn report_rdh_filtered(&self) {
        self.stats_controller_sender_ch
            .send(StatType::RDHsFiltered(1))
            .unwrap();
    }
}

impl<R> ScanCDP for InputScanner<R>
where
    R: ?Sized + BufferedReaderWrapper,
{
    /// Reads the next RDH from file
    /// If a link filter is set, it checks if the RDH matches the chosen link and returns it if it does.
    /// If it doesn't match, it jumps to the next RDH and tries again.
    /// If no link filter is set, it simply returns the RDH.
    #[inline]
    fn load_rdh_cru<T: RDH>(&mut self) -> Result<T, std::io::Error> {
        // If it is the first time we get an RDH, we would already have loaded the initial RDH0
        //  from the input. If so, we use it to create the first RDH.
        let rdh: T = match self.initial_rdh0.is_some() {
            true => RDH::load_from_rdh0(&mut self.reader, self.initial_rdh0.take().unwrap())?,
            false => RDH::load(&mut self.reader)?,
        };
        log::debug!(
            "Loaded RDH at [{:#X}]: \n      {rdh}",
            self.tracker.memory_address_bytes,
            rdh = rdh
        );

        // Set the link ID and report another RDH seen
        let current_link_id = rdh.link_id();
        self.report_rdh_seen();

        // If we haven't seen this link before, report it and add it to the list of unique links
        if !self.unique_links_observed.contains(&current_link_id) {
            self.unique_links_observed.push(current_link_id);
            self.report_link_seen(current_link_id);
        }

        // If we have a link filter set, check if the current link matches the filter
        if let Some(x) = self.link_to_filter.as_ref() {
            // If it matches, return the RDH
            if x.contains(&current_link_id) {
                self.report_rdh_filtered();
                // no jump. current pos -> start of payload
                Ok(rdh)
            } else {
                // If it doesn't match: Set tracker to jump to next RDH and try until we find a matching link or EOF
                log::debug!("Loaded RDH offset to next: {}", rdh.offset_to_next());
                let next_rdh_memory_location = (rdh.offset_to_next() - 64) as i64;
                if next_rdh_memory_location < 0 {
                    self.stats_controller_sender_ch
                        .send(StatType::Fatal(
                            "RDH offset to next is smaller than 64 bytes. This is not possible."
                                .to_string(),
                        ))
                        .unwrap();
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "RDH offset to next is smaller than 64 bytes. This is not possible.",
                    ));
                }
                self.reader
                    .seek_relative(self.tracker.next(rdh.offset_to_next() as u64))?;
                self.load_next_rdh_to_filter()
            }
        } else {
            // No filter set, return the RDH
            // No jump, current position is start of payload
            Ok(rdh)
        }
    }

    /// Reads the next payload from file, using the payload size from the RDH
    #[inline]
    fn load_payload_raw(&mut self, payload_size: usize) -> Result<Vec<u8>, std::io::Error> {
        let mut payload = vec![0; payload_size];
        Read::read_exact(&mut self.reader, &mut payload)?;
        self.report_payload_size(payload_size);
        Ok(payload)
    }
    /// Reads the next CDP from file
    #[inline]
    fn load_cdp<T: RDH>(&mut self) -> Result<(T, Vec<u8>), std::io::Error> {
        log::trace!("Attempting to load CDP - 1. loading RDH");
        let rdh: T = self.load_rdh_cru()?;

        self.tracker.memory_address_bytes += rdh.offset_to_next() as u64;
        // log::trace!(
        //     "Current memory offset: {}, rdh memory offset: {}",
        //     self.tracker.memory_address_bytes,
        //     rdh.offset_to_next()
        // );
        log::trace!("Attempting to load CDP - 2. loading Payload");
        let payload = self.load_payload_raw(rdh.payload_size() as usize)?;
        Ok((rdh, payload))
    }

    fn load_next_rdh_to_filter<T: RDH>(&mut self) -> Result<T, std::io::Error> {
        let links_to_filter = self.link_to_filter.as_ref().unwrap();
        loop {
            let rdh: T = RDH::load(&mut self.reader)?;
            log::debug!("Loaded RDH: \n      {rdh}");
            log::debug!("Loaded RDH offset to next: {}", rdh.offset_to_next());
            let next_rdh_memory_location = rdh.offset_to_next() as i64 - 64;
            if next_rdh_memory_location < 0 {
                self.stats_controller_sender_ch
                    .send(StatType::Fatal(
                        "RDH offset to next is smaller than 64 bytes (size of RDH). This is not possible."
                            .to_string(),
                    ))
                    .unwrap();
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "RDH offset to next is smaller than 64 bytes (size of RDH). This is not possible.",
                ));
            }
            let current_link_id = rdh.link_id();
            self.report_rdh_seen();
            if !self.unique_links_observed.contains(&current_link_id) {
                self.unique_links_observed.push(current_link_id);
                self.report_link_seen(current_link_id);
            }
            if links_to_filter.contains(&current_link_id) {
                self.report_rdh_filtered();
                return Ok(rdh);
            }
            self.reader
                .seek_relative(self.tracker.next(rdh.offset_to_next() as u64))?;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::{fs::File, io::BufReader, path::PathBuf, thread::JoinHandle};

    use crate::{util::stats_controller::Stats, words::rdh::RdhCRUv7, ByteSlice};

    fn setup_scanner_for_file(
        path: &str,
    ) -> (InputScanner<BufReader<std::fs::File>>, JoinHandle<()>) {
        use super::*;
        let config: Opt =
            <Opt as structopt::StructOpt>::from_iter(&["fastpasta", path, "-s", "-f", "0"]);
        let (send_stats_controller_channel, recv_stats_controller_channel): (
            std::sync::mpsc::Sender<StatType>,
            std::sync::mpsc::Receiver<StatType>,
        ) = std::sync::mpsc::channel();

        let thread_stopper = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

        let mut stats_controller =
            Stats::new(&config, recv_stats_controller_channel, thread_stopper);
        let stats_controller_thread = std::thread::spawn(move || {
            stats_controller.run();
        });
        let cfg = std::sync::Arc::new(config);
        let reader = std::fs::OpenOptions::new()
            .read(true)
            .open(cfg.file().to_owned().unwrap())
            .expect("File not found");
        let bufreader = std::io::BufReader::new(reader);

        (
            InputScanner::new(
                cfg,
                Box::new(bufreader),
                MemPosTracker::new(),
                send_stats_controller_channel,
            ),
            stats_controller_thread,
        )
    }

    use super::*;
    use crate::words::rdh::{RdhCRUv6, CORRECT_RDH_CRU_V6, CORRECT_RDH_CRU_V7};
    #[test]
    fn test_load_rdhcruv7_test() {
        let test_data = CORRECT_RDH_CRU_V7;
        println!("Test data: \n       {test_data}");
        let file_name = "test.raw";
        let filepath = PathBuf::from(file_name);
        let mut file = File::create(&filepath).unwrap();
        // Write to file for testing
        file.write_all(test_data.to_byte_slice()).unwrap();

        let stats_handle_super: Option<JoinHandle<()>>;
        {
            let (mut scanner, stats_handle) = setup_scanner_for_file("test.raw");
            stats_handle_super = Some(stats_handle);
            let rdh = scanner.load_rdh_cru::<RdhCRUv7>().unwrap();
            assert_eq!(test_data, rdh);
        }
        stats_handle_super.unwrap().join().unwrap();

        // delete output file
        std::fs::remove_file(filepath).unwrap();
    }

    #[test]
    fn test_load_rdhcruv7_test_unexp_eof() {
        let mut test_data = CORRECT_RDH_CRU_V7;
        test_data.link_id = 100; // Invalid link id
        println!("Test data: \n       {test_data}");
        let file_name = "test.raw";
        let filepath = PathBuf::from(file_name);
        let mut file = File::create(&filepath).unwrap();
        // Write to file for testing
        file.write_all(test_data.to_byte_slice()).unwrap();

        let stats_handle_super: Option<JoinHandle<()>>;
        {
            let (mut scanner, stats_handle) = setup_scanner_for_file("test.raw");
            stats_handle_super = Some(stats_handle);
            let rdh = scanner.load_rdh_cru::<RdhCRUv7>();
            assert!(rdh.is_err());
            assert!(rdh.unwrap_err().kind() == std::io::ErrorKind::UnexpectedEof);
        }
        stats_handle_super.unwrap().join().unwrap();
        // delete output file
        std::fs::remove_file(filepath).unwrap();
    }

    #[test]
    fn test_load_rdhcruv6_test() {
        let mut test_data = CORRECT_RDH_CRU_V6;
        test_data.link_id = 0; // we are filtering for 0
        println!("Test data: \n       {test_data}");
        let file_name = "test.raw";
        let filepath = PathBuf::from(file_name);
        let mut file = File::create(&filepath).unwrap();
        // Write to file for testing
        file.write_all(test_data.to_byte_slice()).unwrap();

        let stats_handle_super: Option<JoinHandle<()>>;
        {
            let (mut scanner, stats_handle) = setup_scanner_for_file("test.raw");
            stats_handle_super = Some(stats_handle);
            let rdh = scanner.load_rdh_cru::<RdhCRUv6>().unwrap();
            assert_eq!(test_data, rdh);
        }
        stats_handle_super.unwrap().join().unwrap();
        // delete output file
        std::fs::remove_file(filepath).unwrap();
    }

    #[test]
    fn test_load_rdhcruv6_test_unexp_eof() {
        let mut test_data = CORRECT_RDH_CRU_V6;
        test_data.link_id = 100; // Invalid link id
        println!("Test data: \n       {test_data}");
        let file_name = "test.raw";
        let filepath = PathBuf::from(file_name);
        let mut file = File::create(&filepath).unwrap();
        // Write to file for testing
        file.write_all(test_data.to_byte_slice()).unwrap();

        let stats_handle_super: Option<JoinHandle<()>>;
        {
            let (mut scanner, stats_handle) = setup_scanner_for_file("test.raw");
            stats_handle_super = Some(stats_handle);
            let rdh = scanner.load_rdh_cru::<RdhCRUv6>();
            assert!(rdh.is_err());
            assert!(rdh.unwrap_err().kind() == std::io::ErrorKind::UnexpectedEof);
        }
        stats_handle_super.unwrap().join().unwrap();
    }
}
