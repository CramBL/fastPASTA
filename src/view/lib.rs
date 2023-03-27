use crate::validators::its_payload_fsm_cont::ItsPayloadFsmContinuous;
use crate::validators::link_validator::preprocess_payload;
use crate::words::lib::RDH;
use crate::{input, stats::stats_controller, util};

use std::io::Write;

#[inline]
pub fn generate_view<T: RDH>(
    view: crate::util::config::View,
    cdp_chunk: input::data_wrapper::CdpChunk<T>,
    send_stats_ch: &std::sync::mpsc::Sender<stats_controller::StatType>,
    its_payload_fsm_cont: &mut ItsPayloadFsmContinuous,
) -> Result<(), std::io::Error> {
    match view {
        util::config::View::Rdh => rdh_view(cdp_chunk)?,
        util::config::View::Hbf => hbf_view(cdp_chunk, send_stats_ch, its_payload_fsm_cont)?,
    }
    Ok(())
}

fn rdh_view<T: RDH>(cdp_chunk: input::data_wrapper::CdpChunk<T>) -> Result<(), std::io::Error> {
    let header_text = crate::words::rdh_cru::RdhCRU::<T>::rdh_header_text_with_indent_to_string(16);
    let mut stdio_lock = std::io::stdout().lock();
    writeln!(stdio_lock, "             {header_text}")?;

    for (rdh, _, mem_pos) in &cdp_chunk {
        writeln!(stdio_lock, "{mem_pos:>8X}:       {rdh}")?;
    }
    Ok(())
}

fn hbf_view<T: RDH>(
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
                let current_word = its_payload_fsm_cont.advance(gbt_word_slice);
                use crate::validators::its_payload_fsm_cont::PayloadWord;
                use crate::words::status_words::util::*;
                let current_mem_pos =
                    calc_current_word_mem_pos(idx, rdh.data_format(), rdh_mem_pos);
                let mem_pos_str = format!("{current_mem_pos:>8X}:");
                let word_slice_str = format_word_slice(gbt_word_slice);
                match current_word {
                    PayloadWord::IHW | PayloadWord::IHW_continuation => {
                        writeln!(stdio_lock, "{mem_pos_str} IHW {word_slice_str}")?;
                    }
                    PayloadWord::TDH => {
                        let trigger_str = tdh_trigger_as_string(gbt_word_slice);
                        let continuation_str = tdh_continuation_as_string(gbt_word_slice);
                        writeln!(
                            stdio_lock,
                            "{mem_pos_str} TDH {word_slice_str} {trigger_str}  {continuation_str}"
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
                        writeln!(
                            stdio_lock,
                            "{mem_pos_str} TDT {word_slice_str} {packet_status_str:>18}",
                        )?;
                    }
                    PayloadWord::DDW0 => {
                        let error_reporting_str = ddw0_error_status_as_string(gbt_word_slice);

                        writeln!(
                            stdio_lock,
                            "{mem_pos_str} DDW {word_slice_str} {error_reporting_str}",
                        )?;
                    }
                    // Ignore these cases
                    PayloadWord::CDW | PayloadWord::DataWord => (),
                }
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
        "Memory    Word{:>37}{:>12}{:>12}",
        "Trig.", "Packet", "Link"
    )?;
    writeln!(
        stdio_lock,
        "Position      {:>36} {:>12}{:>12}\n",
        "type", "status", "ID  "
    )?;
    Ok(())
}

fn print_rdh_hbf_view<T: RDH>(
    rdh: &T,
    rdh_mem_pos: &u64,
    stdio_lock: &mut std::io::StdoutLock,
) -> Result<(), std::io::Error> {
    let trig_str = rdh_trigger_type_as_string(rdh);

    writeln!(
        stdio_lock,
        "{rdh_mem_pos:>8X}: RDH v{}       {trig_str:>28}                      #{:<18}",
        rdh.version(),
        rdh.link_id()
    )?;
    Ok(())
}

const PHT_BIT_MASK: u32 = 0b1_0000;
const SOC_BIT_MASK: u32 = 0b10_0000_0000;
fn rdh_trigger_type_as_string<T: RDH>(rdh: &T) -> String {
    let trigger_type = rdh.trigger_type();
    if trigger_type & PHT_BIT_MASK != 0 {
        String::from("PhT  ")
    } else if trigger_type & SOC_BIT_MASK != 0 {
        String::from("SOC  ")
    } else {
        String::from("Other")
    }
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
