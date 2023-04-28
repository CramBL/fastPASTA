//! Contains the [check_cdp_chunk] function, which iterates over and comnsumes a [`data_wrapper::CdpChunk<T>`], dispatching the data to the correct thread based on the Link ID running an instance of [LinkValidator].
use super::link_validator::LinkValidator;
use crate::{input::data_wrapper, util, words::lib::RDH};
type CdpTuple<T> = (T, Vec<u8>, u64);
/// Iterates over and consumes a [`data_wrapper::CdpChunk<T>`], dispatching the data to the correct thread running an instance of [LinkValidator].
///
/// If a link validator thread does not exist for the link id of the current rdh, a new one is spawned
///
/// Arguments:
/// * `cdp_chunk` - The cdp chunk to be processed
/// * `links` - A vector of link ids that have been seen so far
/// * `link_process_channels` - A vector of producer channels to send data to the link validator threads
/// * `validator_thread_handles` - A vector of handles to the link validator threads
/// * `config` - The config object
/// * `stats_sender_channel` - The producer channel to send stats to the stats controller
pub fn check_cdp_chunk<T: RDH + 'static>(
    cdp_chunk: data_wrapper::CdpChunk<T>,
    links: &mut Vec<u8>,
    link_process_channels: &mut Vec<crossbeam_channel::Sender<CdpTuple<T>>>,
    validator_thread_handles: &mut Vec<std::thread::JoinHandle<()>>,
    config: std::sync::Arc<impl util::lib::Config + 'static>,
    stats_sender_channel: std::sync::mpsc::Sender<crate::stats::stats_controller::StatType>,
) {
    for (rdh, data, mem_pos) in cdp_chunk.into_iter() {
        if let Some(link_index) = links.iter().position(|&x| x == rdh.link_id()) {
            link_process_channels
                .get(link_index)
                .unwrap()
                .send((rdh, data, mem_pos))
                .unwrap();
        } else {
            links.push(rdh.link_id());
            // Create a new link validator thread
            let (mut link_validator, send_channel) =
                LinkValidator::new(&*config.clone(), stats_sender_channel.clone());
            // Add the send channel to the new link validator
            link_process_channels.push(send_channel);

            // Spawn a thread where the newly created link validator will run
            validator_thread_handles.push(
                std::thread::Builder::new()
                    .name(format!("Link {} Validator", rdh.link_id()))
                    .spawn({
                        move || {
                            link_validator.run();
                        }
                    })
                    .expect("Failed to spawn link validator thread"),
            );
            link_process_channels
                .last()
                .unwrap()
                .send((rdh, data, mem_pos))
                .unwrap();
        }
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
/// * `data_format` - The data format of the payload from the RDH, only used to cross check with the detected data format in debug mode
/// Returns:
/// * An iterator over the GBT words
pub fn preprocess_payload(
    payload: &[u8],
    data_format: u8,
) -> Result<impl Iterator<Item = &[u8]>, String> {
    let ff_padding = extract_payload_ff_padding(payload)?;

    // Determine if padding is flavor 0 (6 bytes of 0x00 padding following GBT words) or flavor 1 (no padding)
    let detected_data_format = detect_payload_data_format(payload);
    debug_assert_eq!(data_format, detected_data_format);

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
    use super::*;

    // Test values
    const START_PAYLOAD_FLAVOR_0: [u8; 32] = [
        0xC0, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe0, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x03, 0x1a, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0xE8, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];
    const START_PAYLOAD_FLAVOR_2: [u8; 20] = [
        0x38, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe0, 0x13, 0x08, 0x00, 0x00, 0x00,
        0xD7, 0x39, 0x9B, 0x00, 0xE8,
    ];
    // Flavor 0 has no 0xFF padding, this is just a TDT followed by the 0x00 padding
    const END_PAYLOAD_FLAVOR_0: [u8; 14] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe4, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    // TDT and 0xFF padding
    const END_PAYLOAD_FLAVOR_1: [u8; 14] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe4, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    ];

    #[test]
    fn test_preprocess_payload_flavors() {
        let word_chunk_f0 = preprocess_payload(&START_PAYLOAD_FLAVOR_0, 0).unwrap();
        let word_chunks_f2 = preprocess_payload(&START_PAYLOAD_FLAVOR_2, 2).unwrap();

        let word_count = word_chunk_f0.count();
        let word_count_f2 = word_chunks_f2.count();

        assert_eq!(word_count, 2);
        assert_eq!(word_count_f2, 2);
    }

    #[test]
    fn test_extract_payload_padding() {
        let end_payload_flavor_0_padding =
            extract_payload_ff_padding(&END_PAYLOAD_FLAVOR_0).unwrap();
        let end_payload_flavor_1_padding =
            extract_payload_ff_padding(&END_PAYLOAD_FLAVOR_1).unwrap();

        assert!(end_payload_flavor_0_padding.is_empty());
        assert_eq!(end_payload_flavor_1_padding.len(), 6);
    }

    #[test]
    fn test_detect_payload_data_format() {
        let detected_data_format_f0 = detect_payload_data_format(&START_PAYLOAD_FLAVOR_0);
        let detected_data_format_f2 = detect_payload_data_format(&START_PAYLOAD_FLAVOR_2);

        assert_eq!(detected_data_format_f0, 0);
        assert_eq!(detected_data_format_f2, 2);
    }
}
