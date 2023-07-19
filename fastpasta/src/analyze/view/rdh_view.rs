use alice_protocol_reader::prelude::*;
use std::io::Write;

pub(crate) fn rdh_view<T: RDH>(cdp_chunk: &CdpChunk<T>) -> Result<(), std::io::Error> {
    let header_text = RdhCru::<T>::rdh_header_text_with_indent_to_string(16);
    let mut stdio_lock = std::io::stdout().lock();
    writeln!(stdio_lock, "{header_text}")?;

    for (rdh, _, mem_pos) in cdp_chunk {
        writeln!(stdio_lock, "{mem_pos:>8X}:       {rdh}")?;
    }
    Ok(())
}
