//! Contains the entry point and dispatcher function [generate_view()] for generating data views.
use crate::{
    input, stats::lib::StatType, util,
    validators::its::its_payload_fsm_cont::ItsPayloadFsmContinuous, words::lib::RDH,
};

/// Calls a specific view generator based on the [View][util::config::View] type.
#[inline]
pub fn generate_view<T: RDH>(
    view: crate::util::config::View,
    cdp_chunk: input::data_wrapper::CdpChunk<T>,
    send_stats_ch: &std::sync::mpsc::Sender<StatType>,
    its_payload_fsm_cont: &mut ItsPayloadFsmContinuous,
) -> Result<(), std::io::Error> {
    match view {
        util::config::View::Rdh => super::rdh_view::rdh_view(cdp_chunk)?,
        util::config::View::Hbf => {
            super::hbf_view::hbf_view(cdp_chunk, send_stats_ch, its_payload_fsm_cont)?
        }
    }
    Ok(())
}

const PHT_BIT_MASK: u32 = 0b1_0000;
const SOC_BIT_MASK: u32 = 0b10_0000_0000;
const SOT_BIT_MASK: u32 = 0b1000_0000;
const HB_BIT_MASK: u32 = 0b10;
/// Takes in an RDH and returns a human readable description of the trigger type
///
/// A trigger can be a combination of different types of triggers, so the description is
/// prioritized in terms of what triggers are more significant to understand the trigger type
pub fn rdh_trigger_type_as_string<T: RDH>(rdh: &T) -> String {
    let trigger_type = rdh.trigger_type();
    // Priorities describing the trigger as follows:
    // 1. SOC
    // 2. SOT
    // 3. HB
    // 4. PhT
    if trigger_type & SOC_BIT_MASK != 0 {
        String::from("SOC  ")
    } else if trigger_type & SOT_BIT_MASK != 0 {
        String::from("SOT  ")
    } else if trigger_type & HB_BIT_MASK != 0 {
        String::from("HB   ")
    } else if trigger_type & PHT_BIT_MASK != 0 {
        String::from("PhT  ")
    } else {
        String::from("Other")
    }
}
