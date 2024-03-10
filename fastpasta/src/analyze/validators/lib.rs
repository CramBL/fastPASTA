//! Contains utility functions for preprocessing the payload

use crate::util::*;

#[derive(Debug, PartialEq, Clone, Copy)]
enum DataFormat {
    V0,
    V2,
}

/// Utility function to preprocess the payload and return an iterator over the GBT words
///
/// Consists of the following steps:
/// 1. Extract the end of payload 0xFF padding
/// 2. Determine if padding is flavor 0 (6 bytes of 0x00 padding following GBT words) or flavor 1 (no padding)
/// 3. Split the payload into GBT words sized slices, using chunks_exact to allow more compiler optimizations
///
/// Arguments:
///
/// * `payload` - The payload to be processed
///
/// Returns:
///
/// * An iterator over the GBT words
pub fn preprocess_payload(payload: &[u8]) -> Result<ChunksExact<'_, u8>, String> {
    let ff_padding = extract_payload_ff_padding(payload)?;

    // Determine if padding is flavor 0 (6 bytes of 0x00 padding following GBT words) or flavor 1 (no padding)
    let detected_data_format = detect_payload_data_format(payload);

    let gbt_word_chunks = chunkify_payload(payload, detected_data_format, &ff_padding);
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
fn detect_payload_data_format(payload: &[u8]) -> DataFormat {
    // Using an iterator approach instead of indexing also supports the case where the payload is smaller than 16 bytes or even empty
    if payload
        .iter()
        // Skip the first 10 bytes, meaning the first GBT word
        .skip(10)
        // Take the next 6 bytes
        .take(6)
        // Take bytes while they are equal to 0x00
        .take_while(|&x| *x == 0x00)
        // Count them and check if they are equal to 6
        .count()
        == 6
    {
        DataFormat::V0
    } else {
        DataFormat::V2
    }
}

/// Splits a payload into GBT words sized slices, using chunks_exact to allow more compiler optimizations
fn chunkify_payload<'a>(
    payload: &'a [u8],
    data_format: DataFormat,
    ff_padding: &[&'a u8],
) -> ChunksExact<'a, u8> {
    match data_format {
        DataFormat::V0 => {
            let chunks = payload.chunks_exact(16);
            // If dataformat 0, dividing into 16 byte chunks should cut the payload up with no remainder
            debug_assert!(chunks.remainder().is_empty());
            chunks
        }
        DataFormat::V2 => {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        assert_eq!(detected_data_format_f0, DataFormat::V0);
        assert_eq!(detected_data_format_f2, DataFormat::V2);
    }
}
