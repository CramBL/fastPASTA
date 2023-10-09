use crate::{
    analyze::validators::{its::lib::ItsPayloadWord, lib::preprocess_payload},
    analyze::view::lib::format_word_slice,
};
use alice_protocol_reader::cdp_wrapper::cdp_array::CdpArray;
use alice_protocol_reader::prelude::*;
use std::io::Write;

pub(crate) fn its_readout_frame_view<T: RDH, const CAP: usize>(
    cdp_array: CdpArray<T, CAP>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdio_lock = std::io::stdout().lock();
    super::print_start_of_its_readout_frame_header_text(&mut stdio_lock)?;
    for (rdh, payload, rdh_mem_pos) in cdp_array.into_iter() {
        super::print_rdh_its_readout_frame_view(&rdh, &rdh_mem_pos, &mut stdio_lock)?;
        let gbt_word_chunks = preprocess_payload(&payload)?;
        for (idx, gbt_word) in gbt_word_chunks.enumerate() {
            let word = &gbt_word[..10];
            let mem_pos_str = super::mem_pos_calc_to_string(idx, rdh.data_format(), rdh_mem_pos);
            generate_status_word_view(word, &mem_pos_str, &mut stdio_lock)?;
        }
    }
    Ok(())
}

fn generate_status_word_view(
    word: &[u8],
    mem_pos_str: &str,
    stdio_lock: &mut std::io::StdoutLock,
) -> Result<(), Box<dyn std::error::Error>> {
    match ItsPayloadWord::from_id(word[9]) {
        Ok(word_type) => {
            generate_its_readout_frame_word_view(word_type, word, mem_pos_str, stdio_lock)?
        }
        Err(e) => {
            let word_str = format_word_slice(word);
            let trimmed_mem_pos_str = mem_pos_str.trim();
            log::error!(
                "{trimmed_mem_pos_str} {e}: {:#02X} found in: {word_str}",
                word[9]
            );
        }
    }

    Ok(())
}

/// Generates a human readable view of ITS readout frame words based on the raw word, word type, and memory position.
///
/// Takes:
///     * The word byte slice
///     * The type of PayloadWord from the ITS payload protocol
///     * The memory position of the word
fn generate_its_readout_frame_word_view(
    word_type: crate::analyze::validators::its::lib::ItsPayloadWord,
    gbt_word_slice: &[u8],
    mem_pos_str: &str,
    stdio_lock: &mut std::io::StdoutLock,
) -> Result<(), std::io::Error> {
    use crate::words::its::status_words::util as sw_util;

    let word_slice_str = crate::analyze::view::lib::format_word_slice(gbt_word_slice);
    match word_type {
        // Ignore data words
        ItsPayloadWord::DataWord => (),
        ItsPayloadWord::TDH => {
            let trigger_str = sw_util::tdh_trigger_as_string(gbt_word_slice);
            let continuation_str = sw_util::tdh_continuation_as_string(gbt_word_slice);
            let no_data_str = sw_util::tdh_no_data_as_string(gbt_word_slice);
            let trig_orbit_bc_str = sw_util::tdh_trigger_orbit_bc_as_string(gbt_word_slice);
            writeln!(
                        stdio_lock,
                        "{mem_pos_str} TDH {word_slice_str} {trigger_str}  {continuation_str}        {no_data_str} {trig_orbit_bc_str:>42}"
                    )?;
        }

        ItsPayloadWord::TDT => {
            let packet_status_str = sw_util::tdt_packet_done_as_string(gbt_word_slice);
            let error_reporting_str = sw_util::ddw0_tdt_lane_status_as_string(gbt_word_slice);
            writeln!(
                        stdio_lock,
                        "{mem_pos_str} TDT {word_slice_str} {packet_status_str:>18}                             {error_reporting_str}",
                    )?;
        }
        ItsPayloadWord::IHW => {
            writeln!(stdio_lock, "{mem_pos_str} IHW {word_slice_str}")?;
        }

        ItsPayloadWord::DDW0 => {
            let error_reporting_str = sw_util::ddw0_tdt_lane_status_as_string(gbt_word_slice);

            writeln!(
                            stdio_lock,
                            "{mem_pos_str} DDW {word_slice_str}                                                {error_reporting_str}",
                        )?;
        }
        ItsPayloadWord::CDW => {
            writeln!(
                            stdio_lock,
                            "{mem_pos_str} CDW {word_slice_str}                                                ",
                        )?;
        }
        ItsPayloadWord::IHW_continuation
        | ItsPayloadWord::TDH_continuation
        | ItsPayloadWord::TDH_after_packet_done => {
            unsafe {
                // This function receives only simple types,
                //  as they are coming from ItsPayloadWord::from_id() and not from the FSM that can determine more complex types
                std::hint::unreachable_unchecked()
            }
        }
    }
    Ok(())
}
