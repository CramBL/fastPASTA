//! Contains the [do_payload_checks] which is the entry point for the ITS specific CDP validator
use super::cdp_running::CdpRunningValidator;
use crate::{stats::lib::StatType, util::lib::Config, words::lib::RDH};

/// # Arguments
/// * `cdp_chunk_slice` - A tuple containing the RDH, the payload and the RDH memory position
/// * `send_stats_channel` - The channel to send stats through
/// * `cdp_validator` - The CDP validator to use, which is an ITS specific [CdpRunningValidator]
pub fn do_payload_checks<T: RDH, C: Config>(
    cdp_chunk_slice: (&T, &[u8], u64),
    send_stats_channel: &std::sync::mpsc::Sender<StatType>,
    cdp_validator: &mut CdpRunningValidator<T, C>,
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
