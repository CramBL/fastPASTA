use alice_protocol_reader::cdp_wrapper::cdp_array::CdpArray;
use alice_protocol_reader::prelude::*;
use owo_colors::OwoColorize;
use std::io::Write;

use crate::config::Cfg;
use crate::UtilOpt;

pub(crate) fn rdh_view<T: RDH, const CAP: usize>(
    cdp_array: &CdpArray<T, CAP>,
) -> Result<(), std::io::Error> {
    let mut stdio_lock = std::io::stdout().lock();

    if Cfg::global().disable_styled_views() {
        let header_text = RdhCru::rdh_header_text_with_indent_to_string(16);
        writeln!(stdio_lock, "{header_text}")?;
        for (rdh, _, mem_pos) in cdp_array {
            writeln!(stdio_lock, "{mem_pos:>8X}:       {rdh}")?;
        }
    } else {
        let header_text = RdhCru::rdh_header_styled_text_with_indent_to_string(16);
        writeln!(stdio_lock, "{header_text}")?;
        for (rdh, _, mem_pos) in cdp_array {
            writeln!(
                stdio_lock,
                "{memory_position}       {rdh}",
                memory_position = format_args!("{mem_pos:>8X}:").on_purple().bold()
            )?;
        }
    }

    Ok(())
}
