use crate::{
    analyze::validators::{its::lib::ItsPayloadWord, lib::preprocess_payload},
    analyze::view::lib::format_word_slice,
    input,
    words::lib::RDH,
};
use std::io::Write;

pub(crate) fn its_readout_frame_view<T: RDH>(
    cdp_chunk: input::data_wrapper::CdpChunk<T>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdio_lock = std::io::stdout().lock();
    print_start_of_its_readout_frame_header_text(&mut stdio_lock)?;
    for (rdh, payload, rdh_mem_pos) in cdp_chunk.into_iter() {
        print_rdh_its_readout_frame_view(&rdh, &rdh_mem_pos, &mut stdio_lock)?;
        let gbt_word_chunks = preprocess_payload(&payload)?;
        for (idx, gbt_word) in gbt_word_chunks.enumerate() {
            let word = &gbt_word[..10];
            let mem_pos_str = mem_pos_calc_to_string(idx, rdh.data_format(), rdh_mem_pos);
            generate_status_word_view(word, mem_pos_str, &mut stdio_lock)?;
        }
    }
    Ok(())
}

fn mem_pos_calc_to_string(idx: usize, data_format: u8, rdh_mem_pos: u64) -> String {
    let current_mem_pos = super::lib::calc_current_word_mem_pos(idx, data_format, rdh_mem_pos);
    format!("{current_mem_pos:>8X}:")
}

fn generate_status_word_view(
    word: &[u8],
    mem_pos_str: String,
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

fn print_start_of_its_readout_frame_header_text(
    stdio_lock: &mut std::io::StdoutLock,
) -> Result<(), std::io::Error> {
    writeln!(
        stdio_lock,
        "\nMemory    Word{:>37}{:>12}{:>12}{:>12}{:>12}{:>19}",
        "Trig.", "Packet", "Expect", "Link", "Lane  ", "Trigger  "
    )?;
    writeln!(
        stdio_lock,
        "Position  type{:>36} {:>12}{:>12}{:>12}{:>12}{:>19}\n",
        "type", "status", "Data? ", "ID  ", "faults", "Orbit_BC "
    )?;
    Ok(())
}

fn print_rdh_its_readout_frame_view<T: RDH>(
    rdh: &T,
    rdh_mem_pos: &u64,
    stdio_lock: &mut std::io::StdoutLock,
) -> Result<(), std::io::Error> {
    let trig_str = super::lib::rdh_trigger_type_as_string(rdh);

    writeln!(
        stdio_lock,
        "{rdh_mem_pos:>8X}: RDH v{} stop={}{trig_str:>28}                                #{:<18}",
        rdh.version(),
        rdh.stop_bit(),
        rdh.link_id()
    )?;
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
    mem_pos_str: String,
    stdio_lock: &mut std::io::StdoutLock,
) -> Result<(), std::io::Error> {
    use crate::words::its::status_words::util::*;

    let word_slice_str = crate::analyze::view::lib::format_word_slice(gbt_word_slice);
    match word_type {
        ItsPayloadWord::IHW => {
            writeln!(stdio_lock, "{mem_pos_str} IHW {word_slice_str}")?;
        }
        ItsPayloadWord::TDH => {
            let trigger_str = tdh_trigger_as_string(gbt_word_slice);
            let continuation_str = tdh_continuation_as_string(gbt_word_slice);
            let no_data_str = tdh_no_data_as_string(gbt_word_slice);
            let trig_orbit_bc_str = tdh_trigger_orbit_bc_as_string(gbt_word_slice);
            writeln!(
                            stdio_lock,
                            "{mem_pos_str} TDH {word_slice_str} {trigger_str}  {continuation_str}        {no_data_str} {trig_orbit_bc_str:>42}"
                        )?;
        }

        ItsPayloadWord::TDT => {
            let packet_status_str = tdt_packet_done_as_string(gbt_word_slice);
            let error_reporting_str = ddw0_tdt_lane_status_as_string(gbt_word_slice);
            writeln!(
                            stdio_lock,
                            "{mem_pos_str} TDT {word_slice_str} {packet_status_str:>18}                             {error_reporting_str}",
                        )?;
        }
        ItsPayloadWord::DDW0 => {
            let error_reporting_str = ddw0_tdt_lane_status_as_string(gbt_word_slice);

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
        // Ignore data words
        ItsPayloadWord::DataWord => (),
        ItsPayloadWord::IHW_continuation
        | ItsPayloadWord::TDH_continuation
        | ItsPayloadWord::TDH_after_packet_done => {
            unreachable!("This function should only receive simple types!")
        }
    }
    Ok(())
}
