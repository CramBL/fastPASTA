use crate::util::*;
use io::Write;

pub mod its_readout_frame_data_view;
pub mod its_readout_frame_view;

fn mem_pos_calc_to_string(
    idx: usize,
    data_format: u8,
    rdh_mem_pos: u64,
    disable_styled_view: bool,
) -> String {
    let current_mem_pos = calc_current_word_mem_pos(idx, data_format, rdh_mem_pos);
    if disable_styled_view {
        format!("{current_mem_pos:>8X}:",)
    } else {
        format!(
            "{}",
            format_args!("{current_mem_pos:>8X}:",)
                .white()
                .bg_rgb::<MEM_POS_RED, 0, 0>()
        )
    }
}

pub const RDH_RED: u8 = 50;
pub const MEM_POS_RED: u8 = 10;
pub const TRIG_TYPE_BLUE: u8 = 30;
pub const WORD_TYPE_GREEN: u8 = 20;
pub const PACKET_STATUS_YELLOW_R: u8 = 40;
pub const PACKET_STATUS_YELLOW_G: u8 = 40;
pub const EXPECT_DATA_BLUE: u8 = 30;
pub const LINK_ID_GREEN: u8 = 30;
pub const LANE_FAULTS_RED: u8 = 40;
pub const TRIGGER_ORBIT_BC_YELLOW_R: u8 = 50;
pub const TRIGGER_ORBIT_BC_YELLOW_G: u8 = 50;

fn print_start_of_its_readout_frame_header_text(
    stdio_lock: &mut io::StdoutLock,
    disable_styled_view: bool,
) -> Result<(), io::Error> {
    const MEM_POS_TOP: &str = "Memory  ";
    const MEM_POS_BOT: &str = "Position";
    const WORD_TYPE_TOP: &str = "Word ";
    const WORD_TYPE_BOT: &str = "type ";
    const TRIG_TYPE_TOP: &str = "Trig.";
    const TRIG_TYPE_BOT: &str = "type ";
    const PACKET_STATUS_TOP: &str = "Packet";
    const PACKET_STATUS_BOT: &str = "status";
    const EXPECT_DATA_TOP: &str = "Expect";
    const EXPECT_DATA_BOT: &str = "Data? ";
    const LINK_ID_TOP: &str = "Link";
    const LINK_ID_BOT: &str = "ID  ";
    const LANE_FAULTS_TOP: &str = "Lane  ";
    const LANE_FAULTS_BOT: &str = "faults";
    const TRIGGER_ORBIT_BC_TOP: &str = "Trigger ";
    const TRIGGER_ORBIT_BC_BOT: &str = "Orbit_BC";

    let (top_str, bot_str) = if disable_styled_view {
        (format!("{MEM_POS_TOP}  {WORD_TYPE_TOP}                               {TRIG_TYPE_TOP}       {PACKET_STATUS_TOP}     {EXPECT_DATA_TOP}       {LINK_ID_TOP}      {LANE_FAULTS_TOP}           {TRIGGER_ORBIT_BC_TOP}"),
        format!("{MEM_POS_BOT}  {WORD_TYPE_BOT}                               {TRIG_TYPE_BOT}       {PACKET_STATUS_BOT}     {EXPECT_DATA_BOT}       {LINK_ID_BOT}      {LANE_FAULTS_BOT}           {TRIGGER_ORBIT_BC_BOT}"))
    } else {
        (format!("{mem_pos_top}  {word_type_top}                               {trig_type_top}       {packet_status_top}     {expect_data_top}       {link_id_top}      {lane_faults_top}           {trigger_orbit_bc_top}",
        mem_pos_top = MEM_POS_TOP.bold().white().bg_rgb::<MEM_POS_RED, 0, 0>(),
        word_type_top = WORD_TYPE_TOP.bold().white().bg_rgb::<0, WORD_TYPE_GREEN, 0>(),
        trig_type_top = TRIG_TYPE_TOP.bold().white().bg_rgb::<0, 0, TRIG_TYPE_BLUE>(),
        packet_status_top = PACKET_STATUS_TOP.bold().white().bg_rgb::<PACKET_STATUS_YELLOW_R, PACKET_STATUS_YELLOW_G, 0>(),
        expect_data_top = EXPECT_DATA_TOP.bold().white().bg_rgb::<0, 0, EXPECT_DATA_BLUE>(),
        link_id_top = LINK_ID_TOP.bold().white().bg_rgb::<0, LINK_ID_GREEN, 0>(),
        lane_faults_top = LANE_FAULTS_TOP.bold().white().bg_rgb::<LANE_FAULTS_RED, 0, 0>(),
        trigger_orbit_bc_top = TRIGGER_ORBIT_BC_TOP.bold().white().bg_rgb::<TRIGGER_ORBIT_BC_YELLOW_R, TRIGGER_ORBIT_BC_YELLOW_G, 0>()
    ),
        format!("{mem_pos_bot}  {word_type_bot}                               {trig_type_bot}       {packet_status_bot}     {expect_data_bot}       {link_id_bot}      {lane_faults_bot}           {trigger_orbit_bc_bot}",
        mem_pos_bot = MEM_POS_BOT.bold().white().bg_rgb::<MEM_POS_RED, 0, 0>(),
        word_type_bot = WORD_TYPE_BOT.bold().white().bg_rgb::<0, WORD_TYPE_GREEN, 0>(),
        trig_type_bot = TRIG_TYPE_BOT.bold().white().bg_rgb::<0, 0, TRIG_TYPE_BLUE>(),
        packet_status_bot = PACKET_STATUS_BOT.bold().white().bg_rgb::<PACKET_STATUS_YELLOW_R, PACKET_STATUS_YELLOW_G, 0>(),
        expect_data_bot = EXPECT_DATA_BOT.bold().white().bg_rgb::<0, 0, EXPECT_DATA_BLUE>(),
        link_id_bot = LINK_ID_BOT.bold().white().bg_rgb::<0, LINK_ID_GREEN, 0>(),
        lane_faults_bot = LANE_FAULTS_BOT.bold().white().bg_rgb::<LANE_FAULTS_RED, 0, 0>(),
        trigger_orbit_bc_bot = TRIGGER_ORBIT_BC_BOT.bold().white().bg_rgb::<TRIGGER_ORBIT_BC_YELLOW_R, TRIGGER_ORBIT_BC_YELLOW_G, 0>()
    ))
    };

    writeln!(stdio_lock, "\n{top_str}\n{bot_str}\n",)?;
    Ok(())
}

fn print_rdh_its_readout_frame_view<T: RDH>(
    rdh: &T,
    rdh_mem_pos: u64,
    stdio_lock: &mut StdoutLock,
    disable_styled_view: bool,
) -> Result<(), io::Error> {
    let orbit = rdh.rdh1().orbit; // Packed field

    let rdh_info_row = format!(
        "{mem_pos} {rdh_v} {stop} {stave}{trig} {link} {lane_status}{orbit_bc}",
        mem_pos = format_args!("{:>8X}:", rdh_mem_pos),
        rdh_v = format_args!("RDH v{version}", version = rdh.version()),
        stop = format_args!("stop={}", rdh.stop_bit()),
        stave = format_args!(
            "stave: {:<15}",
            words::its::Stave::from_feeid(rdh.fee_id()).to_string()
        ),
        trig = format_args!("{:<35}", super::lib::rdh_trigger_type_as_string(rdh)),
        link = format_args!("#{:>2}", rdh.link_id().to_string()),
        lane_status = format_args!(
            "{:>14}",
            super::lib::rdh_detector_field_lane_status_as_string(rdh)
        ),
        orbit_bc = format_args!("{orbit:>14}_{bc:>4}", bc = rdh.rdh1().bc()),
    );

    if disable_styled_view {
        writeln!(stdio_lock, "{}", rdh_info_row)?;
    } else {
        writeln!(
            stdio_lock,
            "{}",
            rdh_info_row.white().bold().bg_rgb::<RDH_RED, 0, 0>()
        )?;
    }

    Ok(())
}

pub fn generate_status_word_view(
    word: &[u8],
    mem_pos_str: &str,
    stdio_lock: &mut StdoutLock,
    disable_styled_view: bool,
    display_data_words: bool,
) -> Result<(), Box<dyn error::Error>> {
    match ItsPayloadWord::from_id(word[9]) {
        Ok(word_type) => generate_its_readout_frame_word_view(
            word_type,
            word,
            mem_pos_str,
            stdio_lock,
            disable_styled_view,
            display_data_words,
        )?,
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

const DATA_WORD_BLUE: u8 = 30;
const TDH_GREEN: u8 = 60;
const TDT_GREEN: u8 = 20;
const IHW_BROWN_R: u8 = 128;
const IHW_BROWN_G: u8 = 64;
const DDW_BLUE: u8 = 90;
const CDW_PURPLE_R: u8 = 64;
const CDW_PURPLE_B: u8 = 77;

/// Generates a human readable view of ITS readout frame words based on the raw word, word type, and memory position.
///
/// Takes:
///     * The word byte slice
///     * The type of PayloadWord from the ITS payload protocol
///     * The memory position of the word
fn generate_its_readout_frame_word_view(
    word_type: ItsPayloadWord,
    gbt_word_slice: &[u8],
    mem_pos_str: &str,
    stdio_lock: &mut StdoutLock,
    disable_styled_view: bool,
    display_data_words: bool,
) -> Result<(), io::Error> {
    use crate::words::its::status_words::util as sw_util;

    let word_slice_str = format_word_slice(gbt_word_slice);
    match word_type {
        // Ignore data words
        ItsPayloadWord::DataWord => {
            if display_data_words {
                if disable_styled_view {
                    writeln!(stdio_lock, "{mem_pos_str} DATA {word_slice_str}")?;
                } else {
                    writeln!(
                        stdio_lock,
                        "{mem_pos_str} {}",
                        format_args!("DATA {word_slice_str}")
                            .white()
                            .bg_rgb::<0, 0, DATA_WORD_BLUE>()
                    )?;
                }
            }
        }
        ItsPayloadWord::TDH => {
            let trigger_str = sw_util::tdh_trigger_as_string(gbt_word_slice);
            let continuation_str = sw_util::tdh_continuation_as_string(gbt_word_slice);
            let no_data_str = sw_util::tdh_no_data_as_string(gbt_word_slice);
            let trig_orbit_bc_str = sw_util::tdh_trigger_orbit_bc_as_string(gbt_word_slice);
            let tdh_info_row = format!("TDH {word_slice_str} {trigger_str}  {continuation_str}        {no_data_str} {trig_orbit_bc_str:>42}");

            if disable_styled_view {
                writeln!(stdio_lock, "{mem_pos_str} {tdh_info_row}")?;
            } else {
                writeln!(
                    stdio_lock,
                    "{mem_pos_str} {}",
                    tdh_info_row.white().bg_rgb::<0, TDH_GREEN, 0>()
                )?;
            }
        }

        ItsPayloadWord::TDT => {
            let packet_status_str = sw_util::tdt_packet_done_as_string(gbt_word_slice);
            let error_reporting_str = sw_util::ddw0_tdt_lane_status_as_string(gbt_word_slice);
            if disable_styled_view {
                writeln!(
                    stdio_lock,
                    "{mem_pos_str} TDT {word_slice_str} {packet_status_str:>18}                             {error_reporting_str}",
                )?;
            } else {
                writeln!(
                    stdio_lock,
                    "{mem_pos_str} {}",
                    format_args!("TDT {word_slice_str} {packet_status_str:>18}                             {error_reporting_str}                   ")
                        .white()
                        .bg_rgb::<0, TDT_GREEN, 0>()
                )?;
            }
        }
        ItsPayloadWord::IHW => {
            if disable_styled_view {
                writeln!(stdio_lock, "{mem_pos_str} IHW {word_slice_str}")?;
            } else {
                writeln!(
                    stdio_lock,
                    "{mem_pos_str} {}",
                    format_args!(
                        "IHW {word_slice_str}                                                                          "
                    )
                    .white()
                    .bg_rgb::<IHW_BROWN_R, IHW_BROWN_G, 0>()
                )?;
            }
        }

        ItsPayloadWord::DDW0 => {
            let error_reporting_str = sw_util::ddw0_tdt_lane_status_as_string(gbt_word_slice);

            if disable_styled_view {
                writeln!(
                    stdio_lock,
                    "{mem_pos_str} DDW {word_slice_str}                                                {error_reporting_str}",
                )?;
            } else {
                writeln!(
                    stdio_lock,
                    "{mem_pos_str} {}",
                    format_args!("DDW {word_slice_str}                                                {error_reporting_str}                   ")
                        .white()
                        .bg_rgb::<0, 0, DDW_BLUE>()
                )?;
            }
        }
        ItsPayloadWord::CDW => {
            if disable_styled_view {
                writeln!(
                    stdio_lock,
                    "{mem_pos_str} CDW {word_slice_str}                                                ",
                )?;
            } else {
                writeln!(
                    stdio_lock,
                    "{mem_pos_str} {}",
                    format_args!(
                        "CDW {word_slice_str}                                                                          "
                    )
                    .white()
                    .bg_rgb::<CDW_PURPLE_R, 0, CDW_PURPLE_B>()
                )?;
            }
        }
        ItsPayloadWord::IHW_continuation
        | ItsPayloadWord::TDH_continuation
        | ItsPayloadWord::TDH_after_packet_done => {
            unsafe {
                // This function receives only simple types,
                //  as they are coming from ItsPayloadWord::from_id() and not from the FSM that can determine more complex types
                hint::unreachable_unchecked()
            }
        }
    }
    Ok(())
}
