use crate::analyze::validators::lib::preprocess_payload;
use crate::util::*;

pub(crate) fn its_readout_frame_view<T: RDH, const CAP: usize>(
    cdp_array: &CdpArray<T, CAP>,
    disable_styled_view: bool,
) -> Result<(), Box<dyn error::Error>> {
    let mut stdio_lock = io::stdout().lock();
    super::print_start_of_its_readout_frame_header_text(&mut stdio_lock, disable_styled_view)?;
    for (rdh, payload, rdh_mem_pos) in cdp_array.iter() {
        super::print_rdh_its_readout_frame_view(
            rdh,
            rdh_mem_pos,
            &mut stdio_lock,
            disable_styled_view,
        )?;
        let gbt_word_chunks = preprocess_payload(payload)?;
        for (idx, gbt_word) in gbt_word_chunks.enumerate() {
            let word = &gbt_word[..10];
            let mem_pos_str = super::mem_pos_calc_to_string(
                idx,
                rdh.data_format(),
                rdh_mem_pos,
                disable_styled_view,
            );
            super::generate_status_word_view(
                word,
                &mem_pos_str,
                &mut stdio_lock,
                disable_styled_view,
                false,
            )?;
        }
    }
    Ok(())
}
