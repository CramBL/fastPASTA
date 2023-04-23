//! Contains the [InputScanner], [ScanCDP] trait, and [CdpWrapper] tuple. Responsible for reading and forwarding input data.
//!
//! The [InputScanner] implements the [ScanCDP] trait, and uses the [CdpWrapper] tuple for convenience to wrap an RDH, its payload and its memory position.

use super::bufreader_wrapper::BufferedReaderWrapper;
use super::mem_pos_tracker::MemPosTracker;
use crate::util::lib::Config;
use crate::words::lib::RDH;
use crate::{stats::stats_controller::StatType, words::rdh::Rdh0};
use std::io::Read;

/// Trait for a scanner that reads CDPs from a file or stdin
pub trait ScanCDP {
    /// Loads the next [RDH] from the input and returns it
    fn load_rdh_cru<T: RDH>(&mut self) -> Result<T, std::io::Error>;

    /// Loads the payload in the form of raw bytes from the input and returns it
    ///
    /// The size of the payload is given as an argument.
    fn load_payload_raw(&mut self, payload_size: usize) -> Result<Vec<u8>, std::io::Error>;

    /// Loads the next CDP ([RDH] and payload) from the input and returns it as a [CdpWrapper]
    fn load_cdp<T: RDH>(&mut self) -> Result<CdpWrapper<T>, std::io::Error> {
        let rdh: T = self.load_rdh_cru()?;
        let payload = self.load_payload_raw(rdh.payload_size() as usize)?;
        let mem_pos = self.current_mem_pos();

        Ok(CdpWrapper(rdh, payload, mem_pos))
    }

    /// Loads the next [RDH] that matches the user specified link from the input and returns it
    fn load_next_rdh_to_filter<T: RDH>(&mut self) -> Result<T, std::io::Error>;

    /// Convenience function to return the current memory position in the input stream
    fn current_mem_pos(&self) -> u64;
}

/// Convenience tuple to wrap an [RDH], its payload and memory position.
pub struct CdpWrapper<T: RDH>(pub T, pub Vec<u8>, pub u64);

/// Scans data received through a [BufferedReaderWrapper], tracks the position in memory and sends stats to the stats controller.
///
/// Uses the [Config] to filter for user specified links.
/// Implements [ScanCDP] for a [BufferedReaderWrapper].
pub struct InputScanner<R: ?Sized + BufferedReaderWrapper> {
    reader: Box<R>,
    tracker: MemPosTracker,
    stats_controller_sender_ch: std::sync::mpsc::Sender<StatType>,
    link_to_filter: Option<u8>,
    skip_payload: bool,
    unique_links_observed: Vec<u8>,
    initial_rdh0: Option<Rdh0>,
}

impl<R: ?Sized + BufferedReaderWrapper> InputScanner<R> {
    /// Creates a new [InputScanner] from a [Config], [BufferedReaderWrapper], [MemPosTracker] and a producer channel for [StatType].
    pub fn new(
        config: std::sync::Arc<impl Config>,
        reader: Box<R>,
        tracker: MemPosTracker,
        stats_controller_sender_ch: std::sync::mpsc::Sender<StatType>,
    ) -> Self {
        InputScanner {
            reader,
            tracker,
            stats_controller_sender_ch,
            link_to_filter: config.filter_link(),
            skip_payload: config.skip_payload(),
            unique_links_observed: vec![],
            initial_rdh0: None,
        }
    }
    /// Creates a new [InputScanner] from a [Config], [BufferedReaderWrapper], [MemPosTracker], a producer channel for [StatType] and an initial [Rdh0].
    ///
    /// The [Rdh0] is used to determine the RDH version before instantiating the [InputScanner].
    pub fn new_from_rdh0(
        config: std::sync::Arc<impl Config>,
        reader: Box<R>,
        stats_controller_sender_ch: std::sync::mpsc::Sender<StatType>,
        rdh0: Rdh0,
    ) -> Self {
        InputScanner {
            reader,
            tracker: MemPosTracker::new(),
            stats_controller_sender_ch,
            link_to_filter: config.filter_link(),
            skip_payload: config.skip_payload(),
            unique_links_observed: vec![],
            initial_rdh0: Some(rdh0),
        }
    }
    fn report_rdh_seen(&self) {
        self.stats_controller_sender_ch
            .send(StatType::RDHsSeen(1))
            .expect("Failed to send stats, reiver was dropped")
    }
    fn report_link_seen(&self, link_id: u8) {
        self.stats_controller_sender_ch
            .send(StatType::LinksObserved(link_id))
            .expect("Failed to send stats, reiver was dropped")
    }
    fn report_payload_size(&self, payload_size: u32) {
        self.stats_controller_sender_ch
            .send(StatType::PayloadSize(payload_size))
            .expect("Failed to send stats, reiver was dropped")
    }
    fn report_rdh_filtered(&self) {
        self.stats_controller_sender_ch
            .send(StatType::RDHsFiltered(1))
            .expect("Failed to send stats, reiver was dropped")
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
            "Loaded RDH at [{:#X}]: \n       {rdh}",
            self.tracker.memory_address_bytes,
            rdh = rdh
        );

        // Set the link ID and report another RDH seen
        let current_link_id = rdh.link_id();
        self.report_rdh_seen();
        self.report_payload_size(rdh.payload_size() as u32);

        // If we haven't seen this link before, report it and add it to the list of unique links
        if !self.unique_links_observed.contains(&current_link_id) {
            self.unique_links_observed.push(current_link_id);
            self.report_link_seen(current_link_id);
        }
        sanity_check_offset_next(
            &rdh,
            self.tracker.memory_address_bytes,
            &self.stats_controller_sender_ch,
        )?;
        // If we have a link filter set, check if the current link matches the filter
        if let Some(x) = self.link_to_filter {
            // If it matches, return the RDH
            if x == current_link_id {
                self.report_rdh_filtered();
                // no jump. current pos -> start of payload
                Ok(rdh)
            } else {
                // If it doesn't match: Set tracker to jump to next RDH and try until we find a matching link or EOF
                log::debug!("Loaded RDH offset to next: {}", rdh.offset_to_next());

                self.reader
                    .seek_relative(self.tracker.next(rdh.offset_to_next() as u64))?;
                self.load_next_rdh_to_filter()
            }
        } else {
            // No filter set, return the RDH (nop)
            Ok(rdh)
        }
    }

    /// Reads the next payload from file, using the payload size from the RDH
    #[inline]
    fn load_payload_raw(&mut self, payload_size: usize) -> Result<Vec<u8>, std::io::Error> {
        let mut payload = vec![0; payload_size];
        Read::read_exact(&mut self.reader, &mut payload)?;
        Ok(payload)
    }
    /// Reads the next CDP from file
    #[inline]
    fn load_cdp<T: RDH>(&mut self) -> Result<CdpWrapper<T>, std::io::Error> {
        log::trace!("Attempting to load CDP - 1. loading RDH");
        let loading_at_memory_offset = self.tracker.memory_address_bytes;
        let rdh: T = self.load_rdh_cru()?;

        if self.skip_payload {
            // Only interested in RDHs, seek to next RDH
            self.reader
                .seek_relative(self.tracker.next(rdh.offset_to_next() as u64))?;
        } else {
            self.tracker.memory_address_bytes += rdh.offset_to_next() as u64;
        }

        // If we want the payload, read it, otherwise return a vector that cannot allocate
        let payload = if !self.skip_payload {
            log::trace!("Attempting to load CDP - 2. loading Payload");
            self.load_payload_raw(rdh.payload_size() as usize)?
        } else {
            Vec::with_capacity(0)
        };

        Ok(CdpWrapper(rdh, payload, loading_at_memory_offset))
    }

    fn load_next_rdh_to_filter<T: RDH>(&mut self) -> Result<T, std::io::Error> {
        loop {
            let rdh: T = RDH::load(&mut self.reader)?;
            log::debug!("Loaded RDH: \n      {rdh}");
            log::debug!("Loaded RDH offset to next: {}", rdh.offset_to_next());
            sanity_check_offset_next(
                &rdh,
                self.tracker.memory_address_bytes,
                &self.stats_controller_sender_ch,
            )?;
            let current_link_id = rdh.link_id();
            self.report_rdh_seen();
            if !self.unique_links_observed.contains(&current_link_id) {
                self.unique_links_observed.push(current_link_id);
                self.report_link_seen(current_link_id);
            }
            if self.link_to_filter.unwrap() == current_link_id {
                self.report_rdh_filtered();
                return Ok(rdh);
            }
            self.reader
                .seek_relative(self.tracker.next(rdh.offset_to_next() as u64))?;
        }
    }

    fn current_mem_pos(&self) -> u64 {
        self.tracker.memory_address_bytes
    }
}

// The error is fatal to the input scanner, so parsing input is stopped, but the previously read data is still forwarded for checking etc.
fn sanity_check_offset_next<T: RDH>(
    rdh: &T,
    current_memory_address: u64,
    stats_ch: &std::sync::mpsc::Sender<StatType>,
) -> Result<(), std::io::Error> {
    let current_rdh_offset_to_next = rdh.offset_to_next() as i64;
    let next_rdh_memory_location = current_rdh_offset_to_next - 64;
    if next_rdh_memory_location < 0 {
        let error_string = format!(
            "\n[{current_memory_address:#X}]:\n{rdh_header_text}     {rdh}",
            rdh_header_text = crate::words::rdh_cru::RdhCRU::<crate::words::rdh_cru::V7>::rdh_header_text_with_indent_to_string(5)
        );
        let fatal_error_string = format!(
            "RDH offset to next is {current_rdh_offset_to_next} (less than 64 bytes). {error_string}");
        stats_ch
            .send(StatType::Error(fatal_error_string.clone()))
            .unwrap();
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            fatal_error_string,
        ));
    } else if next_rdh_memory_location > 0x4FFF {
        // VERY HIGH OFFSET
        let error_string = format!(
            "\n[{current_memory_address:#X}]:\n{rdh_header_text}     {rdh}",
            rdh_header_text = crate::words::rdh_cru::RdhCRU::<crate::words::rdh_cru::V7>::rdh_header_text_with_indent_to_string(5)
        );
        let fatal_error_string = format!("RDH offset is larger than 20KB. {error_string}");
        stats_ch
            .send(StatType::Error(fatal_error_string.clone()))
            .unwrap();
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            fatal_error_string,
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::{fs::File, io::BufReader, path::PathBuf};

    use crate::util::config::Opt;
    use crate::util::lib::InputOutput;
    use crate::words::lib::ByteSlice;
    use crate::words::rdh_cru::{RdhCRU, V6, V7};

    fn setup_scanner_for_file(
        path: &str,
    ) -> (
        InputScanner<BufReader<std::fs::File>>,
        std::sync::mpsc::Receiver<StatType>,
    ) {
        use super::*;
        let config: Opt = <Opt as structopt::StructOpt>::from_iter(&[
            "fastpasta",
            path,
            "-f",
            "0",
            "check",
            "sanity",
        ]);
        let (send_stats_controller_channel, recv_stats_controller_channel): (
            std::sync::mpsc::Sender<StatType>,
            std::sync::mpsc::Receiver<StatType>,
        ) = std::sync::mpsc::channel();

        let cfg = std::sync::Arc::new(config);
        let reader = std::fs::OpenOptions::new()
            .read(true)
            .open(cfg.input_file().to_owned().unwrap())
            .expect("File not found");
        let bufreader = std::io::BufReader::new(reader);

        (
            InputScanner::new(
                cfg,
                Box::new(bufreader),
                MemPosTracker::new(),
                send_stats_controller_channel,
            ),
            // Has to be returned so it lives long enough for the test. Otherwise it will be dropped, and inputscanner will panic when trying to report stats.
            recv_stats_controller_channel,
        )
    }

    use super::*;
    use crate::words::rdh_cru::test_data::{CORRECT_RDH_CRU_V6, CORRECT_RDH_CRU_V7};
    #[test]
    fn test_load_rdhcruv7_test() {
        let test_data = CORRECT_RDH_CRU_V7;
        println!("Test data: \n       {test_data}");
        let file_name = "test.raw";
        let filepath = PathBuf::from(file_name);
        let mut file = File::create(&filepath).unwrap();
        // Write to file for testing
        file.write_all(test_data.to_byte_slice()).unwrap();

        {
            let (mut scanner, _rcv_channel) = setup_scanner_for_file("test.raw");
            let rdh = scanner.load_rdh_cru::<RdhCRU<V7>>().unwrap();
            assert_eq!(test_data, rdh);
        }

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

        {
            let (mut scanner, _rcv_channel) = setup_scanner_for_file("test.raw");
            let rdh = scanner.load_rdh_cru::<RdhCRU<V7>>();
            assert!(rdh.is_err());
            assert!(rdh.unwrap_err().kind() == std::io::ErrorKind::UnexpectedEof);
        }

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

        {
            let (mut scanner, _rcv_channel) = setup_scanner_for_file("test.raw");
            let rdh = scanner.load_rdh_cru::<RdhCRU<V6>>().unwrap();
            assert_eq!(test_data, rdh);
        }
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
        let mut file = File::create(filepath).unwrap();
        // Write to file for testing
        file.write_all(test_data.to_byte_slice()).unwrap();

        {
            let (mut scanner, _rcv_channel) = setup_scanner_for_file("test.raw");
            let rdh = scanner.load_rdh_cru::<RdhCRU<V6>>();
            assert!(rdh.is_err());
            assert!(rdh.unwrap_err().kind() == std::io::ErrorKind::UnexpectedEof);
        }
    }
}
