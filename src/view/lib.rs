use crate::validators::its_payload_fsm_cont::ItsPayloadFsmContinuous;
use crate::words::lib::RDH;
use crate::{input, stats::stats_controller, util};

#[inline]
pub fn generate_view<T: RDH>(
    view: crate::util::config::View,
    cdp_chunk: input::data_wrapper::CdpChunk<T>,
    send_stats_ch: &std::sync::mpsc::Sender<stats_controller::StatType>,
    its_payload_fsm_cont: &mut ItsPayloadFsmContinuous,
) -> Result<(), std::io::Error> {
    match view {
        util::config::View::Rdh => super::rdh_view::rdh_view(cdp_chunk)?,
        util::config::View::Hbf => {
            super::hbf_view::hbf_view(cdp_chunk, send_stats_ch, its_payload_fsm_cont)?
        }
    }
    Ok(())
}
