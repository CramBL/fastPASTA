use crate::words::lib::RDH;
use crate::{input, stats::stats_controller, util, words};

#[inline]
pub fn generate_view<T: RDH>(
    view: crate::util::config::View,
    cdp_chunk: input::data_wrapper::CdpChunk<T>,
    send_stats_ch: &std::sync::mpsc::Sender<stats_controller::StatType>,
) {
    match view {
        util::config::View::Rdh => rdh_view(cdp_chunk, send_stats_ch),
    }
}

fn rdh_view<T: RDH>(
    cdp_chunk: input::data_wrapper::CdpChunk<T>,
    send_stats_ch: &std::sync::mpsc::Sender<stats_controller::StatType>,
) {
    let header_text = crate::words::rdh_cru::RdhCRU::<T>::rdh_header_text_with_indent_to_string(16);
    let mut stdio_lock = std::io::stdout().lock();
    use std::io::Write;
    if let Err(e) = writeln!(stdio_lock, "             {header_text}") {
        send_stats_ch
            .send(stats_controller::StatType::Fatal(format!(
                "Error while printing RDH header: {e}"
            )))
            .unwrap();
    }
    for (rdh, _, mem_pos) in &cdp_chunk {
        if let Err(e) = writeln!(stdio_lock, "{mem_pos:>8X}:{rdh}") {
            send_stats_ch
                .send(stats_controller::StatType::Fatal(format!(
                    "Error while printing RDH header: {e}"
                )))
                .unwrap();
        }
    }
}
