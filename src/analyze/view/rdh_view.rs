use crate::input::prelude::*;
use std::io::Write;

pub(crate) fn rdh_view<T: crate::words::lib::RDH>(
    cdp_chunk: CdpChunk<T>,
) -> Result<(), std::io::Error> {
    let header_text = crate::words::rdh_cru::RdhCRU::<T>::rdh_header_text_with_indent_to_string(16);
    let mut stdio_lock = std::io::stdout().lock();
    writeln!(stdio_lock, "{header_text}")?;

    for (rdh, _, mem_pos) in &cdp_chunk {
        writeln!(stdio_lock, "{mem_pos:>8X}:       {rdh}")?;
    }
    Ok(())
}
