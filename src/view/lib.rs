//! Contains the entry point and dispatcher function [generate_view()] for generating data views.
use crate::{
    input, stats::lib::StatType, util,
    validators::its::its_payload_fsm_cont::ItsPayloadFsmContinuous, words::lib::RDH,
};

/// Calls a specific view generator based on the [View][util::config::view::View] type.
#[inline]
pub fn generate_view<T: RDH>(
    view: crate::util::config::view::View,
    cdp_chunk: input::data_wrapper::CdpChunk<T>,
    send_stats_ch: &flume::Sender<StatType>,
    its_payload_fsm_cont: &mut ItsPayloadFsmContinuous,
) -> Result<(), Box<dyn std::error::Error>> {
    use util::config::view::View;
    match view {
        View::Rdh => super::rdh_view::rdh_view(cdp_chunk)?,
        View::Hbf => super::hbf_view::hbf_view(cdp_chunk, send_stats_ch, its_payload_fsm_cont)?,
        View::ItsReadoutFrames => super::its_readout_frame_view::its_readout_frame_view(cdp_chunk)?,
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

/// Calculates the current position in the memory of the current word.
///
/// Current payload position is the first byte after the current RDH
/// The gbt word position relative to the current payload is then:
/// relative_mem_pos = gbt_word_counter * (10 + gbt_word_padding_size_bytes)
/// And the absolute position in the memory is then:
/// gbt_word_mem_pos = payload_mem_pos + relative_mem_pos
#[inline]
pub fn calc_current_word_mem_pos(word_idx: usize, data_format: u8, rdh_mem_pos: u64) -> u64 {
    let gbt_word_padding: u64 = if data_format == 0 {
        6
    } else {
        // Data format 2
        0
    };

    let gbt_word_memory_size_bytes: u64 = 10 + gbt_word_padding;
    let relative_mem_pos = word_idx as u64 * gbt_word_memory_size_bytes;
    relative_mem_pos + rdh_mem_pos + 64
}

/// Simple helper function to format a word slice as a string of hex values
pub fn format_word_slice(word_slice: &[u8]) -> String {
    format!(
        "[{:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}]",
        word_slice[0],
        word_slice[1],
        word_slice[2],
        word_slice[3],
        word_slice[4],
        word_slice[5],
        word_slice[6],
        word_slice[7],
        word_slice[8],
        word_slice[9],
    )
}
