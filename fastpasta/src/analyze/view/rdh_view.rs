use crate::util::*;
use io::Write;

pub(crate) fn rdh_view<T: RDH, const CAP: usize>(
    cdp_array: &CdpArray<T, CAP>,
    disable_styled_view: bool,
) -> Result<(), io::Error> {
    let mut stdio_lock = std::io::stdout().lock();

    if disable_styled_view {
        let header_text = RdhCru::rdh_header_text_with_indent_to_string(11);
        writeln!(stdio_lock, "{header_text}")?;
        for (rdh, _, mem_pos) in cdp_array {
            writeln!(stdio_lock, "{mem_pos:>8X}:  {rdh}")?;
        }
    } else {
        let header_text = RdhCru::rdh_header_styled_text_with_indent_to_string(10);
        writeln!(stdio_lock, "{header_text}")?;
        for (rdh, _, mem_pos) in cdp_array {
            writeln!(
                stdio_lock,
                "{memory_position}{styled_rdh}",
                memory_position = format_args!("{mem_pos:>8X}: ").bg_rgb::<51, 0, 51>().bold(),
                styled_rdh = rdh.to_styled_row_view()
            )?;
        }
    }

    Ok(())
}
