//! Contains the [InputScanner] & [ScanCDP] trait, responsible for reading and forwarding input data.
//!
//! The [InputScanner] implements the [ScanCDP] trait.

use crate::prelude::RdhCru;

use super::bufreader_wrapper::BufferedReaderWrapper;
use super::config::filter::{FilterOpt, FilterTarget};
use super::mem_pos_tracker::MemPosTracker;
use super::rdh::Rdh0;
use super::rdh::{SerdeRdh, RDH};
use super::scan_cdp::ScanCDP;
use super::stats::InputStatType;
use super::stats::Stats;
use std::io::Read;

type CdpTuple<T> = (T, Vec<u8>, u64);

/// Scans data read through a [BufferedReaderWrapper], tracks the position in memory and sends [InputStatType] through the [`flume::Sender<InputStatType>`] channel.
///
/// Uses [FilterOpt] to filter for user specified links.
/// Implements [ScanCDP] for a [BufferedReaderWrapper].
#[derive(Debug)]
pub struct InputScanner<R: ?Sized + BufferedReaderWrapper> {
    reader: Box<R>,
    tracker: MemPosTracker,
    stats_sender_ch: Option<flume::Sender<InputStatType>>,
    filter_target: Option<FilterTarget>,
    skip_payload: bool,
    stats: Option<Stats>,
    initial_rdh0: Option<Rdh0>,
}

impl<R: ?Sized + BufferedReaderWrapper> InputScanner<R> {
    /// Creates a new [InputScanner] from a config that implemenents [FilterOpt], [BufferedReaderWrapper], and a producer channel for [InputStatType].
    pub fn new(
        config: &impl FilterOpt,
        reader: Box<R>,
        stats_sender_ch: Option<flume::Sender<InputStatType>>,
    ) -> Self {
        InputScanner {
            reader,
            tracker: MemPosTracker::new(),
            stats_sender_ch: stats_sender_ch.clone(),
            filter_target: config.filter_target(),
            skip_payload: config.skip_payload(),
            stats: stats_sender_ch.map(Stats::new),
            initial_rdh0: None,
        }
    }
    /// Creates a new [InputScanner] from a config that implemenents [FilterOpt], [BufferedReaderWrapper],  a producer channel for [InputStatType] and an initial [Rdh0].
    ///
    /// The [Rdh0] is used to determine the RDH version before instantiating the [InputScanner].
    pub fn new_from_rdh0(
        config: &impl FilterOpt,
        reader: Box<R>,
        stats_sender_ch: Option<flume::Sender<InputStatType>>,
        rdh0: Rdh0,
    ) -> Self {
        InputScanner {
            reader,
            tracker: MemPosTracker::new(),
            filter_target: config.filter_target(),
            stats_sender_ch: stats_sender_ch.clone(),
            skip_payload: config.skip_payload(),
            stats: stats_sender_ch.map(Stats::new),
            initial_rdh0: Some(rdh0),
        }
    }

    /// Creates a new [InputScanner] with minimal functionality from a [BufferedReaderWrapper].
    ///
    /// Every feature is disabled but the [InputScanner] can still load `CDP`s.
    pub fn minimal(reader: Box<R>) -> Self {
        Self {
            reader,
            tracker: Default::default(),
            stats_sender_ch: Default::default(),
            filter_target: Default::default(),
            skip_payload: Default::default(),
            stats: Default::default(),
            initial_rdh0: Default::default(),
        }
    }

    #[inline]
    fn report(&self, stat: InputStatType) {
        if let Some(stats_sender) = self.stats_sender_ch.as_ref() {
            stats_sender
                .send(stat)
                .expect("Failed to send stats, receiver was dropped")
        }
    }
    #[inline]
    fn report_run_trigger_type<T: RDH>(&self, rdh: &T) {
        let raw_trigger_type = rdh.trigger_type();
        self.report(InputStatType::RunTriggerType(raw_trigger_type));
    }
    #[inline]
    fn collect_rdh_seen_stats(&mut self, rdh: &impl RDH) {
        // Set the link ID and report another RDH seen
        let current_link_id = rdh.link_id();

        if let Some(stat_tracker) = self.stats.as_mut() {
            stat_tracker.rdh_seen();
        }

        // If we haven't seen this link before, report it and add it to the list of unique links
        if let Some(stat_tracker) = self.stats.as_mut() {
            stat_tracker.try_add_link(current_link_id);
        }

        // If the FEE ID has not been seen before, report it and add it to the list of unique FEE IDs
        if let Some(stat_tracker) = self.stats.as_mut() {
            stat_tracker.try_add_fee_id(rdh.fee_id());
        }
    }
    #[inline]
    fn initial_collect_stats(&mut self, rdh: &impl RDH) {
        // Report the trigger type as the RunTriggerType describing the type of run the data is from
        self.report_run_trigger_type(rdh);
        self.report(InputStatType::DataFormat(rdh.data_format()));
        self.report(InputStatType::SystemId(rdh.rdh0().system_id));
    }

    fn seek_to_next_rdh(&mut self, offset_to_next: u16) -> Result<(), std::io::Error> {
        self.reader
            .seek_relative_offset(self.tracker.next(offset_to_next as u64))
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
        let rdh: T = if self.initial_rdh0.is_some() {
            let rdh: T =
                SerdeRdh::load_from_rdh0(&mut self.reader, self.initial_rdh0.take().unwrap())?;

            rdh
        } else {
            SerdeRdh::load(&mut self.reader)?
        };

        if self.current_mem_pos() == 0 {
            // Report general initial stats assumed to be the same for the rest of the data
            self.initial_collect_stats(&rdh);
        }

        // Collect stats
        self.collect_rdh_seen_stats(&rdh);
        sanity_check_offset_next(
            &rdh,
            self.tracker.current_mem_address(),
            self.stats_sender_ch.as_ref(),
        )?;

        // If a filter is set, check if the RDH matches the filter
        let rdh = if let Some(target) = self.filter_target {
            if is_rdh_filter_target(&rdh, target) {
                if let Some(stat_tracker) = self.stats.as_mut() {
                    stat_tracker.rdh_filtered();
                }

                Ok(rdh)
            } else {
                // If it doesn't match: Set tracker to jump to next RDH and try until we find a matching link or EOF
                self.load_next_rdh_to_filter(rdh.offset_to_next(), target)
            }
        } else {
            // No filter set, return the RDH (nop)
            Ok(rdh)
        };

        if let Ok(rdh) = &rdh {
            if let Some(stat_tracker) = self.stats.as_mut() {
                stat_tracker.add_payload_size(rdh.payload_size());
            }
        }
        rdh
    }

    /// Reads the next payload from file, using the payload size from the RDH
    #[inline]
    fn load_payload_raw(&mut self, payload_size: usize) -> Result<Vec<u8>, std::io::Error> {
        let mut payload = Vec::with_capacity(payload_size);

        // The read into raw memory through the raw pointer is safe because we just allocated the capacity
        unsafe {
            let ptr = payload.as_mut_ptr();
            let slice = std::slice::from_raw_parts_mut(ptr, payload_size);

            Read::read_exact(&mut self.reader, slice)?;

            // Safe because we just read into all this memory that we now have initialized to valid data
            payload.set_len(payload_size)
        }

        Ok(payload)
    }
    /// Reads the next CDP from file
    #[inline]
    fn load_cdp<T: RDH>(&mut self) -> Result<CdpTuple<T>, std::io::Error> {
        let loading_at_memory_offset = self.tracker.current_mem_address();
        let rdh: T = self.load_rdh_cru()?;

        if self.skip_payload {
            // Only interested in RDHs, seek to next RDH
            if let Err(e) = self.seek_to_next_rdh(rdh.offset_to_next()) {
                if e.kind() == std::io::ErrorKind::InvalidInput {
                    // Report the error and continue, as this is likely due to an invalid offset in the RDH
                    // But we still want to continue processing the RDH so that we might discover what is wrong with it
                    self.report(InputStatType::Error(
                        format!(
                            "{mem_pos:#X}: [E101] Failed to read payload of size {sz}: {e}",
                            sz = rdh.payload_size(),
                            mem_pos = self.current_mem_pos()
                        )
                        .into(),
                    ));
                } else {
                    return Err(e);
                }
            }
        } else {
            self.tracker.update_mem_address(rdh.offset_to_next() as u64);
        }

        // If we want the payload, read it, otherwise return a vector that cannot allocate
        let payload = if self.skip_payload {
            Vec::with_capacity(0)
        } else {
            match self.load_payload_raw(rdh.payload_size() as usize) {
                Ok(payload) => payload,
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // Report the error and continue. We still want to process the RDH.
                    self.report(InputStatType::Error(
                        format!(
                            "{mem_pos:#X}: [E100] Failed to read payload of size {sz}: {e}",
                            sz = rdh.payload_size(),
                            mem_pos = self.current_mem_pos()
                        )
                        .into(),
                    ));
                    Vec::with_capacity(0)
                }
                Err(e) => return Err(e),
            }
        };

        Ok((rdh, payload, loading_at_memory_offset))
    }

    #[inline]
    fn load_next_rdh_to_filter<T: RDH>(
        &mut self,
        offset_to_next: u16,
        filter_target: FilterTarget,
    ) -> Result<T, std::io::Error> {
        self.seek_to_next_rdh(offset_to_next)?;
        loop {
            let rdh: T = SerdeRdh::load(&mut self.reader)?;
            sanity_check_offset_next(
                &rdh,
                self.tracker.current_mem_address(),
                self.stats_sender_ch.as_ref(),
            )?;
            self.collect_rdh_seen_stats(&rdh);

            if is_rdh_filter_target(&rdh, filter_target) {
                if let Some(stat_tracker) = self.stats.as_mut() {
                    stat_tracker.rdh_filtered();
                }
                return Ok(rdh);
            }
            self.seek_to_next_rdh(rdh.offset_to_next())?;
        }
    }

    fn current_mem_pos(&self) -> u64 {
        self.tracker.current_mem_address()
    }
}

impl<R> Drop for InputScanner<R>
where
    R: ?Sized + BufferedReaderWrapper,
{
    fn drop(&mut self) {
        if let Some(mut stat_tracker) = self.stats.take() {
            stat_tracker.flush_stats();
        }
    }
}

// Check if the RDH matches the filter target
#[inline]
fn is_rdh_filter_target(rdh: &impl RDH, target: FilterTarget) -> bool {
    match target {
        FilterTarget::Link(id) => rdh.link_id() == id,
        FilterTarget::Fee(id) => rdh.fee_id() == id,
        FilterTarget::ItsLayerStave(fee_id) => is_match_feeid_layer_stave(rdh.fee_id(), fee_id),
    }
}

#[inline]
fn is_match_feeid_layer_stave(a_fee_id: u16, b_fee_id: u16) -> bool {
    let layer_stave_mask: u16 = 0b0111_0000_0011_1111;
    (a_fee_id & layer_stave_mask) == (b_fee_id & layer_stave_mask)
}

// The error is fatal to the input scanner, so parsing input is stopped, but the previously read data is still forwarded for checking etc.
#[inline]
fn sanity_check_offset_next<T: RDH>(
    rdh: &T,
    current_memory_address: u64,
    stats_ch: Option<&flume::Sender<InputStatType>>,
) -> Result<(), std::io::Error> {
    let next_rdh_memory_location = rdh.offset_to_next() as i64 - 64;
    // If the offset is not between 0 and 10 KB it is invalid
    if !(0..=10_000).contains(&next_rdh_memory_location) {
        // Invalid offset: Negative or very high
        let fatal_err = invalid_rdh_offset(rdh, current_memory_address, next_rdh_memory_location);

        if let Some(stats_ch) = stats_ch.as_ref() {
            stats_ch
                .send(InputStatType::Fatal(fatal_err.clone().into_boxed_str()))
                .unwrap();
        }
        let fatal_io_error = std::io::Error::new(std::io::ErrorKind::InvalidData, fatal_err);
        return Err(fatal_io_error);
    }
    Ok(())
}

#[inline]
fn invalid_rdh_offset<T: RDH>(rdh: &T, current_memory_address: u64, offset_to_next: i64) -> String {
    let error_string = format!(
        "\n[{current_memory_address:#X}]:\n{rdh_header_text}     {rdh}",
        rdh_header_text = RdhCru::rdh_header_text_with_indent_to_string(5)
    );
    format!("RDH offset to next is {offset_to_next}. {error_string}")
}

#[cfg(test)]
mod tests {
    use super::super::config::mock_config::MockConfig;
    use super::super::prelude::test_data::{CORRECT_RDH_CRU_V6, CORRECT_RDH_CRU_V7};
    use super::super::rdh::ByteSlice;
    use super::*;
    use crate::prelude::RDH_CRU;
    use flume::Receiver;
    use pretty_assertions::assert_eq;
    use std::{io::BufReader, path::PathBuf};
    use temp_dir::TempDir;

    fn setup_scanner_for_file(
        path: &PathBuf,
    ) -> (
        InputScanner<BufReader<std::fs::File>>,
        flume::Receiver<InputStatType>,
    ) {
        use super::*;
        let config = MockConfig {
            filter_link: Some(0),
            ..Default::default()
        };
        let (controller_send, controller_recv): (
            flume::Sender<InputStatType>,
            flume::Receiver<InputStatType>,
        ) = flume::unbounded();

        let reader = std::fs::OpenOptions::new()
            .read(true)
            .open(path)
            .expect("File not found");
        let bufreader = std::io::BufReader::new(reader);

        (
            InputScanner::new(&config, Box::new(bufreader), Some(controller_send)),
            // Has to be returned so it lives long enough for the test. Otherwise it will be dropped, and inputscanner will panic when trying to report stats.
            controller_recv,
        )
    }

    #[test]
    fn test_load_rdhcruv7_test() {
        let tmp_d = TempDir::new().unwrap();
        let test_file = tmp_d.child("test.raw");
        let test_data = CORRECT_RDH_CRU_V7;
        println!("Test data: \n       {test_data}");
        // Write to file for testing
        std::fs::write(&test_file, test_data.to_byte_slice()).unwrap();

        let stats_recv: Receiver<InputStatType> = {
            let (mut scanner, recv_chan) = setup_scanner_for_file(&test_file);
            let rdh = scanner.load_rdh_cru::<RdhCru>().unwrap();
            assert_eq!(test_data, rdh);
            recv_chan
        };
        assert!(!stats_recv.is_empty(), "stats_recv was empty!");
    }

    #[test]
    fn test_load_rdhcruv7_test_unexp_eof() {
        let mut test_data = CORRECT_RDH_CRU_V7;
        test_data.link_id = 100; // Invalid link id
        println!("Test data: \n       {test_data}");

        let tmp_d = TempDir::new().unwrap();
        let test_file = tmp_d.child("test.raw");
        std::fs::write(&test_file, test_data.to_byte_slice()).unwrap();

        let stats_recv: Receiver<InputStatType> = {
            let (mut scanner, recv_chan) = setup_scanner_for_file(&test_file);
            let rdh = scanner.load_rdh_cru::<RdhCru>();
            assert!(rdh.is_err());
            assert_eq!(rdh.unwrap_err().kind(), std::io::ErrorKind::UnexpectedEof);
            recv_chan
        };
        assert!(!stats_recv.is_empty(), "stats_recv was empty!");
    }

    #[test]
    fn test_load_rdhcruv6_test() {
        let mut test_data = CORRECT_RDH_CRU_V6;
        test_data.link_id = 0; // we are filtering for 0
        println!("Test data: \n       {test_data}");

        let tmp_d = TempDir::new().unwrap();
        let test_file = tmp_d.child("test.raw");
        std::fs::write(&test_file, test_data.to_byte_slice()).unwrap();

        let stats_recv: Receiver<InputStatType> = {
            let (mut scanner, recv_chan) = setup_scanner_for_file(&test_file);
            let rdh = scanner.load_rdh_cru::<RdhCru>().unwrap();
            assert_eq!(test_data, rdh);
            recv_chan
        };
        assert!(!stats_recv.is_empty(), "stats_recv was empty!");
    }

    #[test]
    fn test_load_rdhcruv6_test_unexp_eof() {
        let mut test_data = CORRECT_RDH_CRU_V6;
        test_data.link_id = 100; // Invalid link id
        println!("Test data: \n       {test_data}");

        let tmp_d = TempDir::new().unwrap();
        let test_file = tmp_d.child("test.raw");
        std::fs::write(&test_file, test_data.to_byte_slice()).unwrap();

        let stats_recv: Receiver<InputStatType> = {
            let (mut scanner, recv_chan) = setup_scanner_for_file(&test_file);

            let rdh = scanner.load_rdh_cru::<RdhCru>();
            assert!(rdh.is_err());
            assert_eq!(rdh.unwrap_err().kind(), std::io::ErrorKind::UnexpectedEof);
            recv_chan
        };
        assert!(!stats_recv.is_empty(), "stats_recv was empty!");
    }

    #[test]
    fn test_attempt_read_payload_at_eof() {
        // Testing that the unsafe reading of payload is handled correctly in the case where the RDH indicates a payload but there's no data, just EOF.
        let test_data = CORRECT_RDH_CRU_V7;

        let tmp_dir = TempDir::new().unwrap();
        let test_file = tmp_dir.child("test.raw");
        std::fs::write(&test_file, test_data.to_byte_slice()).unwrap();
        let reader = std::fs::OpenOptions::new()
            .read(true)
            .open(test_file)
            .expect("File not found");
        let bufreader = std::io::BufReader::new(reader);
        let mut input_scanner = InputScanner::minimal(Box::new(bufreader));

        let rdh = input_scanner.load_rdh_cru::<RdhCru>().unwrap();
        let payload = input_scanner.load_payload_raw(rdh.payload_size().into());

        assert!(payload.is_err());
    }

    #[test]
    fn test_bad_payload_value() {
        // Testing that the unsafe reading of payload is handled correctly in the case where the RDH indicates a payload but there's only a partial payload before reaching EOF.
        let test_data = CORRECT_RDH_CRU_V7;

        let tmp_dir = TempDir::new().unwrap();
        let test_file = tmp_dir.child("test.raw");
        // Write the RDH and a bit of a payload
        std::fs::write(
            &test_file,
            [
                test_data.to_byte_slice(),
                &[0xe8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            ]
            .concat(),
        )
        .unwrap();

        let reader = std::fs::OpenOptions::new()
            .read(true)
            .open(test_file)
            .expect("File not found");
        let bufreader = std::io::BufReader::new(reader);
        let mut input_scanner = InputScanner::minimal(Box::new(bufreader));

        let rdh = input_scanner.load_rdh_cru::<RdhCru>().unwrap();
        let payload = input_scanner.load_payload_raw(rdh.payload_size().into());

        assert!(payload.is_err());
    }
}
