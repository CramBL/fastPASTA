//! Contains the [InputScanner] & [ScanCDP] trait, responsible for reading and forwarding input data.
//!
//! The [InputScanner] implements the [ScanCDP] trait.

use super::bufreader_wrapper::BufferedReaderWrapper;
use super::mem_pos_tracker::MemPosTracker;
use crate::config::filter::{FilterOpt, FilterTarget};
use crate::config::inputoutput::InputOutputOpt;
use crate::stats::{StatType, SystemId};
use crate::words::its::is_match_feeid_layer_stave;
use crate::words::lib::{SerdeRdh, RDH};
use crate::words::rdh::Rdh0;
use std::io::Read;

type CdpTuple<T> = (T, Vec<u8>, u64);

/// Trait for a scanner that reads CDPs from a file or stdin
pub trait ScanCDP {
    /// Loads the next [RDH] from the input and returns it
    fn load_rdh_cru<T: RDH>(&mut self) -> Result<T, std::io::Error>;

    /// Loads the payload in the form of raw bytes from the input and returns it
    ///
    /// The size of the payload is given as an argument.
    fn load_payload_raw(&mut self, payload_size: usize) -> Result<Vec<u8>, std::io::Error>;

    /// Loads the next CDP ([RDH] and payload) from the input and returns it as a ([RDH], [`Vec<u8>`], [u64]) tuple.
    fn load_cdp<T: RDH>(&mut self) -> Result<CdpTuple<T>, std::io::Error> {
        let rdh: T = self.load_rdh_cru()?;
        let payload = self.load_payload_raw(rdh.payload_size() as usize)?;
        let mem_pos = self.current_mem_pos();

        Ok((rdh, payload, mem_pos))
    }

    /// Loads the next [RDH] that matches the user specified filter target from the input and returns it
    fn load_next_rdh_to_filter<T: RDH>(
        &mut self,
        offset_to_next: u16,
        target: FilterTarget,
    ) -> Result<T, std::io::Error>;

    /// Convenience function to return the current memory position in the input stream
    fn current_mem_pos(&self) -> u64;
}

/// Scans data read through a [BufferedReaderWrapper], tracks the position in memory and sends [StatType] through the [`flume::Sender<StatType>`] channel.
///
/// Uses [FilterOpt] to filter for user specified links.
/// Implements [ScanCDP] for a [BufferedReaderWrapper].
pub struct InputScanner<R: ?Sized + BufferedReaderWrapper> {
    reader: Box<R>,
    tracker: MemPosTracker,
    stats_controller_sender_ch: flume::Sender<StatType>,
    filter_target: Option<FilterTarget>,
    skip_payload: bool,
    unique_links_observed: Vec<u8>,
    unique_feeids_observed: Vec<u16>,
    initial_rdh0: Option<Rdh0>,
}

impl<R: ?Sized + BufferedReaderWrapper> InputScanner<R> {
    /// Creates a new [InputScanner] from a config that implemenents [FilterOpt] & [InputOutputOpt], [BufferedReaderWrapper], and a producer channel for [StatType].
    pub fn new(
        config: &(impl FilterOpt + InputOutputOpt),
        reader: Box<R>,
        stats_controller_sender_ch: flume::Sender<StatType>,
    ) -> Self {
        InputScanner {
            reader,
            tracker: MemPosTracker::new(),
            stats_controller_sender_ch,
            filter_target: config.filter_target(),
            skip_payload: config.skip_payload(),
            unique_links_observed: vec![],
            unique_feeids_observed: vec![],
            initial_rdh0: None,
        }
    }
    /// Creates a new [InputScanner] from a config that implemenents [FilterOpt] & [InputOutputOpt], [BufferedReaderWrapper],  a producer channel for [StatType] and an initial [Rdh0].
    ///
    /// The [Rdh0] is used to determine the RDH version before instantiating the [InputScanner].
    pub fn new_from_rdh0(
        config: &(impl FilterOpt + InputOutputOpt),
        reader: Box<R>,
        stats_controller_sender_ch: flume::Sender<StatType>,
        rdh0: Rdh0,
    ) -> Self {
        InputScanner {
            reader,
            tracker: MemPosTracker::new(),
            filter_target: config.filter_target(),
            stats_controller_sender_ch,
            skip_payload: config.skip_payload(),
            unique_links_observed: vec![],
            unique_feeids_observed: vec![],
            initial_rdh0: Some(rdh0),
        }
    }

    fn report(&self, stat: StatType) {
        self.stats_controller_sender_ch
            .send(stat)
            .expect("Failed to send stats, receiver was dropped")
    }
    fn report_run_trigger_type<T: RDH>(&self, rdh: &T) {
        let raw_trigger_type = rdh.trigger_type();
        let run_trigger_type_str = crate::analyze::view::lib::rdh_trigger_type_as_string(rdh);
        self.report(StatType::RunTriggerType((
            raw_trigger_type,
            run_trigger_type_str,
        )));
    }
    fn collect_rdh_seen_stats(&mut self, rdh: &impl RDH) {
        // Set the link ID and report another RDH seen
        let current_link_id = rdh.link_id();
        self.report(StatType::RDHSeen);

        // If we haven't seen this link before, report it and add it to the list of unique links
        if !self.unique_links_observed.contains(&current_link_id) {
            self.unique_links_observed.push(current_link_id);
            self.report(StatType::LinksObserved(current_link_id));
        }
        // If the FEE ID has not been seen before, report it and add it to the list of unique FEE IDs
        if !self.unique_feeids_observed.contains(&rdh.fee_id()) {
            self.unique_feeids_observed.push(rdh.fee_id());
            self.report(StatType::FeeId(rdh.fee_id()));
        }
    }
    fn initial_collect_stats(&mut self, rdh: &impl RDH) -> Result<(), std::io::Error> {
        // Report the trigger type as the RunTriggerType describing the type of run the data is from
        self.report_run_trigger_type(rdh);
        self.report(StatType::DataFormat(rdh.data_format()));
        let observed_sys_id = match SystemId::from_system_id(rdh.rdh0().system_id) {
            Ok(id) => id,
            Err(e) => {
                log::error!("Failed to parse system ID: {e}");
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Failed to parse system ID",
                ));
            }
        };
        log::info!("{observed_sys_id} detected");
        self.report(StatType::SystemId(observed_sys_id));
        Ok(())
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
        //  from the input. If so, we use it to create the first RDH, and record some stats
        //  that should only be recorded once.
        let rdh: T = if self.initial_rdh0.is_some() {
            let rdh: T =
                SerdeRdh::load_from_rdh0(&mut self.reader, self.initial_rdh0.take().unwrap())?;
            // Report general initial stats assumed to be the same for the rest of the data
            self.initial_collect_stats(&rdh)?;
            rdh
        } else {
            SerdeRdh::load(&mut self.reader)?
        };
        log::debug!(
            "Loaded RDH at [{mem_pos:#X}]: \n       {rdh}",
            mem_pos = self.tracker.current_mem_address()
        );

        // Collect stats
        self.collect_rdh_seen_stats(&rdh);
        sanity_check_offset_next(
            &rdh,
            self.tracker.current_mem_address(),
            &self.stats_controller_sender_ch,
        )?;

        // If a filter is set, check if the RDH matches the filter
        let rdh = if let Some(target) = self.filter_target {
            if is_rdh_filter_target(&rdh, target) {
                self.report(StatType::RDHFiltered);

                Ok(rdh)
            } else {
                // If it doesn't match: Set tracker to jump to next RDH and try until we find a matching link or EOF
                log::debug!("Loaded RDH offset to next: {}", rdh.offset_to_next());
                self.load_next_rdh_to_filter(rdh.offset_to_next(), target)
            }
        } else {
            // No filter set, return the RDH (nop)
            Ok(rdh)
        };

        if let Ok(rdh) = &rdh {
            self.report(StatType::PayloadSize(rdh.payload_size() as u32));
        }
        rdh
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
    fn load_cdp<T: RDH>(&mut self) -> Result<CdpTuple<T>, std::io::Error> {
        log::trace!("Attempting to load CDP - 1. loading RDH");
        let loading_at_memory_offset = self.tracker.current_mem_address();
        let rdh: T = self.load_rdh_cru()?;

        if self.skip_payload {
            // Only interested in RDHs, seek to next RDH
            self.reader
                .seek_relative(self.tracker.next(rdh.offset_to_next() as u64))?;
        } else {
            self.tracker.update_mem_address(rdh.offset_to_next() as u64);
        }

        // If we want the payload, read it, otherwise return a vector that cannot allocate
        let payload = if !self.skip_payload {
            self.load_payload_raw(rdh.payload_size() as usize)?
        } else {
            Vec::with_capacity(0)
        };

        Ok((rdh, payload, loading_at_memory_offset))
    }

    fn load_next_rdh_to_filter<T: RDH>(
        &mut self,
        offset_to_next: u16,
        filter_target: FilterTarget,
    ) -> Result<T, std::io::Error> {
        self.reader
            .seek_relative(self.tracker.next(offset_to_next as u64))?;
        loop {
            let rdh: T = SerdeRdh::load(&mut self.reader)?;
            sanity_check_offset_next(
                &rdh,
                self.tracker.current_mem_address(),
                &self.stats_controller_sender_ch,
            )?;
            self.collect_rdh_seen_stats(&rdh);

            if is_rdh_filter_target(&rdh, filter_target) {
                self.report(StatType::RDHFiltered);
                return Ok(rdh);
            }
            self.reader
                .seek_relative(self.tracker.next(rdh.offset_to_next() as u64))?;
        }
    }

    fn current_mem_pos(&self) -> u64 {
        self.tracker.current_mem_address()
    }
}

// Check if the RDH matches the filter target
fn is_rdh_filter_target(rdh: &impl RDH, target: FilterTarget) -> bool {
    match target {
        FilterTarget::Link(id) => rdh.link_id() == id,
        FilterTarget::Fee(id) => rdh.fee_id() == id,
        FilterTarget::ItsLayerStave(fee_id) => is_match_feeid_layer_stave(rdh.fee_id(), fee_id),
    }
}

// The error is fatal to the input scanner, so parsing input is stopped, but the previously read data is still forwarded for checking etc.
fn sanity_check_offset_next<T: RDH>(
    rdh: &T,
    current_memory_address: u64,
    stats_ch: &flume::Sender<StatType>,
) -> Result<(), std::io::Error> {
    let next_rdh_memory_location = rdh.offset_to_next() as i64 - 64;
    // If the offset is not between 0 and 10 KB it is invalid
    if !(0..=10_000).contains(&next_rdh_memory_location) {
        // Invalid offset: Negative or very high
        let fatal_err = invalid_rdh_offset(rdh, current_memory_address, next_rdh_memory_location);
        stats_ch.send(StatType::Error(fatal_err.clone())).unwrap();
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            fatal_err,
        ));
    }
    Ok(())
}

fn invalid_rdh_offset<T: RDH>(rdh: &T, current_memory_address: u64, offset_to_next: i64) -> String {
    use crate::words::rdh_cru::{RdhCRU, V7};
    let error_string = format!(
        "\n[{current_memory_address:#X}]:\n{rdh_header_text}     {rdh}",
        rdh_header_text = RdhCRU::<V7>::rdh_header_text_with_indent_to_string(5)
    );
    format!("RDH offset to next is {offset_to_next}. {error_string}")
}

#[cfg(test)]
mod tests {
    use crate::config::Cfg;
    use crate::words::lib::ByteSlice;
    use crate::words::rdh_cru::{RdhCRU, V6, V7};
    use clap::Parser;
    use pretty_assertions::assert_eq;
    use std::io::Write;
    use std::{fs::File, io::BufReader, path::PathBuf};

    fn setup_scanner_for_file(
        path: &str,
    ) -> (
        InputScanner<BufReader<std::fs::File>>,
        flume::Receiver<StatType>,
    ) {
        use super::*;
        let config: Cfg = <Cfg>::parse_from(["fastpasta", path, "-f", "0", "check", "sanity"]);
        let (send_stats_controller_channel, recv_stats_controller_channel): (
            flume::Sender<StatType>,
            flume::Receiver<StatType>,
        ) = flume::unbounded();

        let cfg = config;
        let reader = std::fs::OpenOptions::new()
            .read(true)
            .open(cfg.input_file().to_owned().unwrap())
            .expect("File not found");
        let bufreader = std::io::BufReader::new(reader);

        (
            InputScanner::new(&cfg, Box::new(bufreader), send_stats_controller_channel),
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
            assert_eq!(rdh.unwrap_err().kind(), std::io::ErrorKind::UnexpectedEof);
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
            assert_eq!(rdh.unwrap_err().kind(), std::io::ErrorKind::UnexpectedEof);
        }
    }
}
