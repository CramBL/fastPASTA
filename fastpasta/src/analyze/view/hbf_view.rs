use crate::analyze::validators::its::its_payload_fsm_cont;
use crate::analyze::validators::its::its_payload_fsm_cont::ItsPayloadFsmContinuous;
use crate::analyze::validators::its::lib::ItsPayloadWord;
use crate::analyze::validators::lib::preprocess_payload;
use crate::stats::StatType;
use alice_protocol_reader::prelude::*;
use std::io::Write;

pub(crate) fn hbf_view<T: RDH>(
    cdp_chunk: CdpChunk<T>,
    send_stats_ch: &flume::Sender<StatType>,
    its_payload_fsm_cont: &mut ItsPayloadFsmContinuous,
) -> Result<(), std::io::Error> {
    let mut stdio_lock = std::io::stdout().lock();
    print_start_of_hbf_header_text(&mut stdio_lock)?;
    for (rdh, payload, rdh_mem_pos) in cdp_chunk.into_iter() {
        print_rdh_hbf_view(&rdh, &rdh_mem_pos, &mut stdio_lock)?;

        let gbt_word_chunks = match preprocess_payload(&payload) {
            Ok(gbt_word_chunks) => Some(gbt_word_chunks),
            Err(e) => {
                send_stats_ch.send(StatType::Error(e)).unwrap();
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
                            ItsPayloadWord::TDH
                        }
                        its_payload_fsm_cont::AmbigiousError::DW_or_TDT_CDW => {
                            log::warn!("The ID of the current word did not match an expected ID. Treating it as a Data Word (not displayed), but it could be incorrect!");
                            ItsPayloadWord::DataWord
                        }
                        its_payload_fsm_cont::AmbigiousError::DDW0_or_TDH_IHW => {
                            log::warn!("The ID of the current word did not match an expected ID. Displaying it as DDW0, but it could be incorrect!");
                            ItsPayloadWord::DDW0
                        }
                    },
                };
                let current_mem_pos =
                    super::lib::calc_current_word_mem_pos(idx, rdh.data_format(), rdh_mem_pos);
                let mem_pos_str = format!("{current_mem_pos:>8X}:");
                generate_hbf_word_view(
                    current_word_type,
                    gbt_word_slice,
                    &mem_pos_str,
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

fn generate_hbf_word_view(
    word_type: crate::analyze::validators::its::lib::ItsPayloadWord,
    gbt_word_slice: &[u8],
    mem_pos_str: &str,
    stdio_lock: &mut std::io::StdoutLock,
) -> Result<(), std::io::Error> {
    use crate::words::its::status_words::util::*;

    let word_slice_str = crate::analyze::view::lib::format_word_slice(gbt_word_slice);
    match word_type {
        ItsPayloadWord::IHW | ItsPayloadWord::IHW_continuation => {
            writeln!(stdio_lock, "{mem_pos_str} IHW {word_slice_str}")?;
        }
        ItsPayloadWord::TDH | ItsPayloadWord::TDH_after_packet_done => {
            let trigger_str = tdh_trigger_as_string(gbt_word_slice);
            let continuation_str = tdh_continuation_as_string(gbt_word_slice);
            let no_data_str = tdh_no_data_as_string(gbt_word_slice);
            writeln!(
                            stdio_lock,
                            "{mem_pos_str} TDH {word_slice_str} {trigger_str}  {continuation_str}        {no_data_str}"
                        )?;
        }
        ItsPayloadWord::TDH_continuation => {
            let trigger_str = tdh_trigger_as_string(gbt_word_slice);
            let continuation_str = tdh_continuation_as_string(gbt_word_slice);
            writeln!(
                stdio_lock,
                "{mem_pos_str} TDH {word_slice_str} {trigger_str}  {continuation_str}"
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
        // Ignore these cases
        ItsPayloadWord::CDW | ItsPayloadWord::DataWord => (),
    }
    Ok(())
}
