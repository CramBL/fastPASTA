//! Contains the [do_payload_checks] which is the entry point for the ITS specific CDP validator
use super::cdp_running::CdpRunningValidator;
use crate::config::prelude::*;
use crate::stats::StatType;
use alice_protocol_reader::prelude::FilterOpt;
use alice_protocol_reader::prelude::RDH;

/// # Arguments
/// * `cdp` - A tuple containing the RDH, the payload and the RDH memory position
/// * `stats_send_chan` - The channel to send stats through
/// * `cdp_validator` - The CDP validator to use, which is an ITS specific [CdpRunningValidator]
pub fn do_payload_checks<T: RDH, C: ChecksOpt + FilterOpt + CustomChecksOpt>(
    cdp: (&T, &[u8], u64),
    stats_send_chan: &flume::Sender<StatType>,
    cdp_validator: &mut CdpRunningValidator<T, C>,
) {
    let (rdh, payload, rdh_mem_pos) = cdp;
    cdp_validator.set_current_rdh(rdh, rdh_mem_pos);
    match crate::analyze::validators::lib::preprocess_payload(payload) {
        Ok(gbt_word_chunks) => gbt_word_chunks.for_each(|gbt_word| {
            cdp_validator.check(&gbt_word[..10]); // Take 10 bytes as flavor 0 would have additional 6 bytes of padding
        }),
        Err(e) => {
            stats_send_chan
                .send(StatType::Error(
                    format!("{rdh_mem_pos:#X}: Payload error following RDH at this location: {e}")
                        .into(),
                ))
                .unwrap();
            cdp_validator.reset_fsm();
        }
    }
}

#[allow(non_camel_case_types)] // An exception to the Rust naming convention, for these words that are already acronyms
/// ITS Payload word types
#[derive(Debug, Clone, Copy)]
pub enum ItsPayloadWord {
    /// ITS Header Word
    IHW,
    /// ITS Header Word in continuation mode
    IHW_continuation,
    /// Trigger Data Header
    TDH,
    /// Trigger Data Header in continuation mode
    TDH_continuation,
    /// Trigger Data Header succeeding a TDT with packet done flag set
    TDH_after_packet_done,
    /// Trigger Data Trailer
    TDT,
    /// Calibration Data Word
    CDW,
    /// Data
    DataWord,
    /// Diagnostic Data Word 0
    DDW0,
}

impl ItsPayloadWord {
    /// Takes in the ID of an ITS Payload word, and returns the type as an enum if it matches any.
    ///
    /// Only returns simple types, i.e. not the continuation types etc.
    pub fn from_id(word_id: u8) -> Result<Self, String> {
        match word_id {
            0x20..=0x28 | 0x40..=0x46 | 0x48..=0x4E | 0x50..=0x56 | 0x58..=0x5E => {
                Ok(ItsPayloadWord::DataWord)
            }
            0xE8 => Ok(ItsPayloadWord::TDH),
            0xF0 => Ok(ItsPayloadWord::TDT),
            0xE0 => Ok(ItsPayloadWord::IHW),
            0xE4 => Ok(ItsPayloadWord::DDW0),
            0xF8 => Ok(ItsPayloadWord::CDW),
            _ => Err("Unknown ITS Payload Word ID".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::test_util::MockConfig;
    use alice_protocol_reader::prelude::test_data::CORRECT_RDH_CRU_V7;
    use alice_protocol_reader::prelude::*;
    use std::sync::OnceLock;

    static CFG_TEST_DO_PAYLOAD_CHECKS: OnceLock<MockConfig> = OnceLock::new();

    #[test]
    fn test_do_payload_checks_bad_payload() {
        let mut mock_config = MockConfig::new();
        mock_config.check = Some(CheckCommands::All {
            system: Some(System::ITS),
        });
        CFG_TEST_DO_PAYLOAD_CHECKS.set(mock_config).unwrap();

        let (stats_send_chan, stats_recv_chan) = flume::unbounded();

        let mut cdp_validator: CdpRunningValidator<RdhCru, MockConfig> = CdpRunningValidator::new(
            CFG_TEST_DO_PAYLOAD_CHECKS.get().unwrap(),
            stats_send_chan.clone(),
        );
        let rdh = CORRECT_RDH_CRU_V7;
        let payload = vec![0x3D; 100];
        let rdh_mem_pos = 0;
        let cdp_slice = (&rdh, payload.as_slice(), rdh_mem_pos);

        do_payload_checks(cdp_slice, &stats_send_chan, &mut cdp_validator);

        // Receive and check stats
        while let Ok(stats) = stats_recv_chan.try_recv() {
            // the payload is only made up of 0x3D, so there should be errors, and all mentioning `3D`
            assert!(stats.to_string().contains("3D"));
            println!("Stats: {stats:?}")
        }
    }
}
