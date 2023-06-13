use crate::words::lib::RDH;
use std::io::Write;

pub mod its_readout_frame_data_view;
pub mod its_readout_frame_view;

fn mem_pos_calc_to_string(idx: usize, data_format: u8, rdh_mem_pos: u64) -> String {
    let current_mem_pos = super::lib::calc_current_word_mem_pos(idx, data_format, rdh_mem_pos);
    format!("{current_mem_pos:>9X}:")
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
    let orbit = rdh.rdh1().orbit;
    let orbit_bc_str = format!("{orbit}_{bc:>4}", bc = rdh.rdh1().bc());

    writeln!(
        stdio_lock,
        "{rdh_mem_pos:>8X}: RDH v{} stop={}{trig_str:>28}                                #{}   {orbit_bc_str:>31}",
        rdh.version(),
        rdh.stop_bit(),
        rdh.link_id()
    )?;
    Ok(())
}
