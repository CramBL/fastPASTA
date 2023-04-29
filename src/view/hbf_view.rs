use crate::input;
use crate::stats::stats_controller;
use crate::validators::its::its_payload_fsm_cont::{self, ItsPayloadFsmContinuous, PayloadWord};
use crate::validators::lib::preprocess_payload;
use crate::words::lib::RDH;
use std::io::Write;

pub(crate) fn hbf_view<T: RDH>(
    cdp_chunk: input::data_wrapper::CdpChunk<T>,
    send_stats_ch: &std::sync::mpsc::Sender<stats_controller::StatType>,
    its_payload_fsm_cont: &mut ItsPayloadFsmContinuous,
) -> Result<(), std::io::Error> {
    let mut stdio_lock = std::io::stdout().lock();
    print_start_of_hbf_header_text(&mut stdio_lock)?;
    for (rdh, payload, rdh_mem_pos) in cdp_chunk.into_iter() {
        print_rdh_hbf_view(&rdh, &rdh_mem_pos, &mut stdio_lock)?;

        let gbt_word_chunks = match preprocess_payload(&payload, rdh.data_format()) {
            Ok(gbt_word_chunks) => Some(gbt_word_chunks),
            Err(e) => {
                send_stats_ch
                    .send(stats_controller::StatType::Error(e))
                    .unwrap();
                its_payload_fsm_cont.reset_fsm();
                None
            }
        };

        if let Some(gbt_words) = gbt_word_chunks {
            for (idx, gbt_word) in gbt_words.enumerate() {
                let gbt_word_slice = &gbt_word[..10];
                // Advance the FSM to find out how to display the current GBT word
                let current_word_type = match its_payload_fsm_cont.advance(gbt_word_slice) {
                    Ok(word) => word,
                    // If the ID is not among the valid IDs for the current state, display a warning and attempt to handle it.
                    Err(ambigious_word) => match ambigious_word {
                        its_payload_fsm_cont::AmbigiousError::TDH_or_DDW0 => {
                            log::warn!("The ID of the current word did not match an expected ID. Displaying it as TDH, but it could be incorrect!");
                            PayloadWord::TDH
                        }
                        its_payload_fsm_cont::AmbigiousError::DW_or_TDT_CDW => {
                            log::warn!("The ID of the current word did not match an expected ID. Treating it as a Data Word (not displayed), but it could be incorrect!");
                            PayloadWord::DataWord
                        }
                        its_payload_fsm_cont::AmbigiousError::DDW0_or_TDH_IHW => {
                            log::warn!("The ID of the current word did not match an expected ID. Displaying it as DDW0, but it could be incorrect!");
                            PayloadWord::DDW0
                        }
                    },
                };
                let current_mem_pos =
                    calc_current_word_mem_pos(idx, rdh.data_format(), rdh_mem_pos);
                let mem_pos_str = format!("{current_mem_pos:>8X}:");
                generate_payload_word_view(
                    gbt_word_slice,
                    current_word_type,
                    mem_pos_str,
                    &mut stdio_lock,
                )?;
            }
        }
    }
    Ok(())
}

fn print_start_of_hbf_header_text(
    stdio_lock: &mut std::io::StdoutLock,
) -> Result<(), std::io::Error> {
    writeln!(
        stdio_lock,
        "\nMemory    Word{:>37}{:>12}{:>12}{:>12}{:>12}",
        "Trig.", "Packet", "Expect", "Link", "Lane  "
    )?;
    writeln!(
        stdio_lock,
        "Position  type{:>36} {:>12}{:>12}{:>12}{:>12}\n",
        "type", "status", "Data? ", "ID  ", "faults"
    )?;
    Ok(())
}

fn print_rdh_hbf_view<T: RDH>(
    rdh: &T,
    rdh_mem_pos: &u64,
    stdio_lock: &mut std::io::StdoutLock,
) -> Result<(), std::io::Error> {
    let trig_str = super::lib::rdh_trigger_type_as_string(rdh);

    writeln!(
        stdio_lock,
        "{rdh_mem_pos:>8X}: RDH v{}       {trig_str:>28}                                #{:<18}",
        rdh.version(),
        rdh.link_id()
    )?;
    Ok(())
}

/// Calculates the current position in the memory of the current word.
///
/// Current payload position is the first byte after the current RDH
/// The gbt word position then relative to the current payload is then:
/// relative_mem_pos = gbt_word_counter * (10 + gbt_word_padding_size_bytes)
/// And the absolute position in the memory is then:
/// gbt_word_mem_pos = payload_mem_pos + relative_mem_pos
#[inline]
fn calc_current_word_mem_pos(word_idx: usize, data_format: u8, rdh_mem_pos: u64) -> u64 {
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

fn generate_payload_word_view(
    gbt_word_slice: &[u8],
    word_type: its_payload_fsm_cont::PayloadWord,
    mem_pos_str: String,
    stdio_lock: &mut std::io::StdoutLock,
) -> Result<(), std::io::Error> {
    use crate::words::its::status_words::util::*;

    let word_slice_str = format_word_slice(gbt_word_slice);
    match word_type {
        PayloadWord::IHW | PayloadWord::IHW_continuation => {
            writeln!(stdio_lock, "{mem_pos_str} IHW {word_slice_str}")?;
        }
        PayloadWord::TDH | PayloadWord::TDH_after_packet_done => {
            let trigger_str = tdh_trigger_as_string(gbt_word_slice);
            let continuation_str = tdh_continuation_as_string(gbt_word_slice);
            let no_data_str = tdh_no_data_as_string(gbt_word_slice);
            writeln!(
                            stdio_lock,
                            "{mem_pos_str} TDH {word_slice_str} {trigger_str}  {continuation_str}        {no_data_str}"
                        )?;
        }
        PayloadWord::TDH_continuation => {
            let trigger_str = tdh_trigger_as_string(gbt_word_slice);
            let continuation_str = tdh_continuation_as_string(gbt_word_slice);
            writeln!(
                stdio_lock,
                "{mem_pos_str} TDH {word_slice_str} {trigger_str}  {continuation_str}"
            )?;
        }
        PayloadWord::TDT => {
            let packet_status_str = tdt_packet_done_as_string(gbt_word_slice);
            let error_reporting_str = ddw0_tdt_lane_status_as_string(gbt_word_slice);
            writeln!(
                            stdio_lock,
                            "{mem_pos_str} TDT {word_slice_str} {packet_status_str:>18}                             {error_reporting_str}",
                        )?;
        }
        PayloadWord::DDW0 => {
            let error_reporting_str = ddw0_tdt_lane_status_as_string(gbt_word_slice);

            writeln!(
                            stdio_lock,
                            "{mem_pos_str} DDW {word_slice_str}                                                {error_reporting_str}",
                        )?;
        }
        // Ignore these cases
        PayloadWord::CDW | PayloadWord::DataWord => (),
    }
    Ok(())
}

fn format_word_slice(word_slice: &[u8]) -> String {
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
