use crate::words::its::Stave;
use alice_protocol_reader::prelude::RDH;
use owo_colors::OwoColorize;
use std::io::Write;

pub mod its_readout_frame_data_view;
pub mod its_readout_frame_view;

fn mem_pos_calc_to_string(idx: usize, data_format: u8, rdh_mem_pos: u64) -> String {
    let current_mem_pos = super::lib::calc_current_word_mem_pos(idx, data_format, rdh_mem_pos);
    format!("{current_mem_pos:>8X}:")
}

pub const RDH_RED: u8 = 50;
pub const MEM_POS_RED: u8 = 80;
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
    stdio_lock: &mut std::io::StdoutLock,
    disable_styled_view: bool,
) -> Result<(), std::io::Error> {
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
    stdio_lock: &mut std::io::StdoutLock,
    disable_styled_view: bool,
) -> Result<(), std::io::Error> {
    let orbit = rdh.rdh1().orbit; // Packed field

    let rdh_info_row = format!(
        "{mem_pos} {rdh_v} {stop} {stave}{trig} {link} {lane_status}{orbit_bc}",
        mem_pos = format_args!("{:>8X}:", rdh_mem_pos),
        rdh_v = format_args!("RDH v{version}", version = rdh.version()),
        stop = format_args!("stop={}", rdh.stop_bit()),
        stave = format_args!("stave: {:<15}", Stave::from_feeid(rdh.fee_id()).to_string()),
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
