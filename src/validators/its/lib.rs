//! Contains the [do_payload_checks] which is the entry point for the ITS specific CDP validator
use super::cdp_running::CdpRunningValidator;
use crate::{stats::lib::StatType, words::lib::RDH};

/// # Arguments
/// * `cdp_chunk_slice` - A tuple containing the RDH, the payload and the RDH memory position
/// * `send_stats_channel` - The channel to send stats through
/// * `cdp_validator` - The CDP validator to use, which is an ITS specific [CdpRunningValidator]
pub fn do_payload_checks<T: RDH>(
    cdp_chunk_slice: (&T, &[u8], u64),
    send_stats_channel: &std::sync::mpsc::Sender<StatType>,
    cdp_validator: &mut CdpRunningValidator<T>,
) {
    let (rdh, payload, rdh_mem_pos) = cdp_chunk_slice;
    cdp_validator.set_current_rdh(rdh, rdh_mem_pos);
    match crate::validators::lib::preprocess_payload(payload) {
        Ok(gbt_word_chunks) => gbt_word_chunks.for_each(|gbt_word| {
            cdp_validator.check(&gbt_word[..10]); // Take 10 bytes as flavor 0 would have additional 6 bytes of padding
        }),
        Err(e) => {
            send_stats_channel.send(StatType::Error(e)).unwrap();
            cdp_validator.reset_fsm();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::words::rdh_cru::{test_data::CORRECT_RDH_CRU_V7, *};

    #[test]
    fn test_do_payload_checks_bad_payload() {
        let (send_stats_ch, rcv_stats_ch) = std::sync::mpsc::channel();

        let mut cdp_validator: CdpRunningValidator<RdhCRU<V7>> =
            CdpRunningValidator::_new_no_cfg(send_stats_ch.clone());
        let rdh = CORRECT_RDH_CRU_V7;
        let payload = vec![0x3D; 100];
        let rdh_mem_pos = 0;
        let cdp_chunk_slice = (&rdh, payload.as_slice(), rdh_mem_pos);

        do_payload_checks(cdp_chunk_slice, &send_stats_ch, &mut cdp_validator);

        // Receive and check stats
        while let Ok(stats) = rcv_stats_ch.try_recv() {
            match stats {
                _ => {
                    // the payload is only made up of 0x3D, so there should be errors, and all mentioning `3D`
                    assert!(stats.to_string().contains("3D"));
                    println!("Stats: {:?}", stats)
                }
            }
        }
    }
}
