//! Contains the [ValidatorDispatcher], that manages [LinkValidator]s and iterates over and consumes a [`CdpChunk<T>`], dispatching the data to the correct thread based on the Link ID running an instance of [LinkValidator].
use super::link_validator::LinkValidator;
use crate::config::prelude::*;
use crate::input::prelude::CdpChunk;
use crate::stats::StatType;

use crate::words::lib::RDH;
type CdpTuple<T> = (T, Vec<u8>, u64);

/// The [ValidatorDispatcher] is responsible for creating and managing the [LinkValidator] threads.
///
/// It receives a [`CdpChunk<T>`] and dispatches the data to the correct thread running an instance of [LinkValidator].
pub struct ValidatorDispatcher<T: RDH, C: Config + 'static> {
    processors: Vec<DispatchId>,
    process_channels: Vec<crossbeam_channel::Sender<CdpTuple<T>>>,
    validator_thread_handles: Vec<std::thread::JoinHandle<()>>,
    stats_sender: flume::Sender<StatType>,
    global_config: &'static C,
}

#[derive(PartialEq, Clone, Copy)]
struct DispatchId(u16);

impl<T: RDH + 'static, C: Config + 'static> ValidatorDispatcher<T, C> {
    /// Create a new ValidatorDispatcher from a Config and a stats sender channel
    pub fn new(global_config: &'static C, stats_sender: flume::Sender<StatType>) -> Self {
        Self {
            processors: Vec::new(),
            process_channels: Vec::new(),
            validator_thread_handles: Vec::new(),
            stats_sender,
            global_config,
        }
    }

    /// Iterates over and consumes a [`CdpChunk<T>`], dispatching the data to the correct thread running an instance of [LinkValidator].
    ///
    /// If a link validator thread does not exist for the link id of the current rdh, a new one is spawned
    pub fn dispatch_cdp_chunk(&mut self, cdp_chunk: CdpChunk<T>) {
        // Iterate over the CDP chunk
        cdp_chunk.into_iter().for_each(|(rdh, data, mem_pos)| {
            // Dispatch by FEE ID if system targeted for checks is ITS Stave (gonna be a lot of data to parse for each stave!)
            let id = if self.global_config.check().is_some_and(|c| {
                if let CheckCommands::All { system } = c {
                    system.is_some_and(|s| s == System::ITS_Stave)
                } else {
                    false
                }
            }) {
                // Dispatch by FEE ID which will effectively dispatch by link AND stave
                DispatchId(rdh.fee_id())
            } else {
                // Dispatch by link ID
                DispatchId(rdh.link_id() as u16)
            };

            self.dispatch_by_id(rdh, data, mem_pos, id);
        });
    }

    fn init_validator(&mut self, id: DispatchId) -> LinkValidator<T, C> {
        // Add a new ID to the list of processors
        self.processors.push(id);

        // Create a new link validator thread to handle a new ID that should be processed
        let (link_validator, send_channel) =
            LinkValidator::<T, C>::new(self.global_config, self.stats_sender.clone());

        // Add the send channel to the new link validator
        self.process_channels.push(send_channel);

        link_validator
    }

    fn dispatch_by_id(&mut self, rdh: T, data: Vec<u8>, mem_pos: u64, id: DispatchId) {
        // Check if the ID to dispatch by is already in the list of processors
        if let Some(index) = self.processors.iter().position(|&proc_id| proc_id == id) {
            // If the ID was found, use its index to send the data through the correct link validator's channel
            self.process_channels
                .get(index)
                .unwrap()
                .send((rdh, data, mem_pos))
                .unwrap();
        } else {
            // If the ID wasn't found, make a new validator to handle that ID
            let mut validator = self.init_validator(id);

            // Spawn a thread where the newly created link validator will run
            self.validator_thread_handles.push(
                std::thread::Builder::new()
                    .name(format!("Validator #{}", id.0))
                    .spawn({
                        move || {
                            validator.run();
                        }
                    })
                    .expect("Failed to spawn link validator thread"),
            );
            // Send the data through the newly created link validator's channel, by taking the last element of the vector
            self.process_channels
                .last()
                .unwrap()
                .send((rdh, data, mem_pos))
                .unwrap();
        }
    }

    /// Disconnects all the link validator's receiver channels and joins all link validator threads
    pub fn join(&mut self) {
        self.process_channels.clear();
        self.validator_thread_handles.drain(..).for_each(|handle| {
            handle.join().expect("Failed to join a validator thread");
        });
    }
}

/// Utility function to preprocess the payload and return an iterator over the GBT words
///
/// Consists of the following steps:
/// 1. Extract the end of payload 0xFF padding
/// 2. Determine if padding is flavor 0 (6 bytes of 0x00 padding following GBT words) or flavor 1 (no padding)
/// 3. Split the payload into GBT words sized slices, using chunks_exact to allow more compiler optimizations
///
/// Arguments:
/// * `payload` - The payload to be processed
/// Returns:
/// * An iterator over the GBT words
pub fn preprocess_payload(payload: &[u8]) -> Result<impl Iterator<Item = &[u8]>, String> {
    let ff_padding = extract_payload_ff_padding(payload)?;

    // Determine if padding is flavor 0 (6 bytes of 0x00 padding following GBT words) or flavor 1 (no padding)
    let detected_data_format = detect_payload_data_format(payload);

    let gbt_word_chunks = chunkify_payload(payload, detected_data_format, ff_padding);
    Ok(gbt_word_chunks)
}

/// Retrieve end of payload 0xFF padding, if it is more than 15 bytes, return an error
fn extract_payload_ff_padding(payload: &[u8]) -> Result<Vec<&u8>, String> {
    let ff_padding = payload
        .iter()
        .rev()
        .take_while(|&x| *x == 0xFF)
        .collect::<Vec<_>>();
    // Exceeds the maximum padding of 15 bytes that is required to pad to 16 bytes
    if ff_padding.len() > 15 {
        return Err(format!("End of payload 0xFF padding is {} bytes, exceeding max of 15 bytes: Skipping current payload",
        ff_padding.len()));
    }
    Ok(ff_padding)
}

/// Determine if padding is flavor 0 (6 bytes of 0x00 padding following GBT words) or flavor 1 (no padding)
fn detect_payload_data_format(payload: &[u8]) -> u8 {
    // Using an iterator approach instead of indexing also supports the case where the payload is smaller than 16 bytes or even empty
    if payload
    .iter() // Create an iterator over the payload
    .skip(10) // Skip the first 10 bytes, meaning the first GBT word
    .take(6) // Take the next 6 bytes
    .take_while(|&x| *x == 0x00) // Take bytes while they are equal to 0x00
    .count() // Count them and check if they are equal to 6
    == 6
    {
        log::trace!("Data format 0 detected");
        0
    } else {
        log::trace!("Data format 2 detected");
        2
    }
}

/// Splits a payload into GBT words sized slices, using chunks_exact to allow more compiler optimizations
fn chunkify_payload<'a>(
    payload: &'a [u8],
    data_format: u8,
    ff_padding: Vec<&'a u8>,
) -> std::slice::ChunksExact<'a, u8> {
    match data_format {
        0 => {
            let chunks = payload.chunks_exact(16);
            // If dataformat 0, dividing into 16 byte chunks should cut the payload up with no remainder
            debug_assert!(chunks.remainder().is_empty());
            chunks
        }
        2 => {
            // If dataformat 2, and the padding is more than 9 bytes, padding will be processed as a GBT word, therefor exclude it from the slice
            //    Before calling chunks_exact
            if ff_padding.len() > 9 {
                let last_idx_before_padding = payload.len() - ff_padding.len();
                let chunks = payload[..last_idx_before_padding].chunks_exact(10);
                debug_assert!(chunks.remainder().is_empty());
                chunks
            } else {
                // Simply divide into 10 byte chunks and assert that the remainder is padding bytes
                let chunks = payload.chunks_exact(10);
                debug_assert!(chunks.remainder().iter().all(|&x| x == 0xFF)); // Asserts that the payload padding is 0xFF
                chunks
            }
        }
        _ => unreachable!("Invalid data format"),
    }
}

#[cfg(test)]
mod tests {
    use crate::config::check::CheckCommands;
    use crate::config::test_util::MockConfig;
    use crate::input::prelude::*;
    use crate::words::its::test_payloads::*;
    use crate::words::rdh_cru::test_data::CORRECT_RDH_CRU_V7;
    use crate::words::rdh_cru::{RdhCRU, V7};
    use std::sync::OnceLock;

    use super::*;

    static CFG_TEST_DISPACTER: OnceLock<MockConfig> = OnceLock::new();

    #[test]
    fn test_dispacter() {
        let mut cfg = MockConfig::new();
        cfg.check = Some(CheckCommands::Sanity { system: None });
        CFG_TEST_DISPACTER.set(cfg).unwrap();

        let mut disp: ValidatorDispatcher<RdhCRU<V7>, MockConfig> =
            ValidatorDispatcher::new(CFG_TEST_DISPACTER.get().unwrap(), flume::unbounded().0);

        let cdp_tuple: CdpTuple<RdhCRU<V7>> = (CORRECT_RDH_CRU_V7, vec![0; 100], 0);

        let mut cdp_chunk = CdpChunk::new();
        cdp_chunk.push_tuple(cdp_tuple);

        disp.dispatch_cdp_chunk(cdp_chunk);

        disp.join();
    }

    #[test]
    fn test_preprocess_payload_flavors() {
        let word_chunk_f0 = preprocess_payload(&START_PAYLOAD_FLAVOR_0).unwrap();
        let word_chunks_f2 = preprocess_payload(&START_PAYLOAD_FLAVOR_2).unwrap();

        let word_count = word_chunk_f0.count();
        let word_count_f2 = word_chunks_f2.count();

        assert_eq!(word_count, 2);
        assert_eq!(word_count_f2, 2);
    }

    #[test]
    fn test_extract_payload_padding() {
        let end_payload_flavor_0_padding =
            extract_payload_ff_padding(&END_PAYLOAD_FLAVOR_0).unwrap();
        let end_payload_flavor_2_padding =
            extract_payload_ff_padding(&END_PAYLOAD_FLAVOR_2).unwrap();

        assert!(end_payload_flavor_0_padding.is_empty());
        assert_eq!(end_payload_flavor_2_padding.len(), 6);
    }

    #[test]
    fn test_detect_payload_data_format() {
        let detected_data_format_f0 = detect_payload_data_format(&START_PAYLOAD_FLAVOR_0);
        let detected_data_format_f2 = detect_payload_data_format(&START_PAYLOAD_FLAVOR_2);

        assert_eq!(detected_data_format_f0, 0);
        assert_eq!(detected_data_format_f2, 2);
    }
}
