//! Contains the [check_cdp_chunk] function, which iterates over and comnsumes a [`data_wrapper::CdpChunk<T>`], dispatching the data to the correct thread based on the Link ID running an instance of [LinkValidator].
use super::link_validator::LinkValidator;
use crate::{input::data_wrapper, util, words::lib::RDH};
type CdpTuple<T> = (T, Vec<u8>, u64);
/// Iterates over and consumes a [`data_wrapper::CdpChunk<T>`], dispatching the data to the correct thread running an instance of [LinkValidator].
///
/// If a link validator thread does not exist for the link id of the current rdh, a new one is spawned
///
/// Arguments:
/// * `cdp_chunk` - The cdp chunk to be processed
/// * `links` - A vector of link ids that have been seen so far
/// * `link_process_channels` - A vector of producer channels to send data to the link validator threads
/// * `validator_thread_handles` - A vector of handles to the link validator threads
/// * `config` - The config object
/// * `stats_sender_channel` - The producer channel to send stats to the stats controller
pub fn check_cdp_chunk<T: RDH + 'static>(
    cdp_chunk: data_wrapper::CdpChunk<T>,
    links: &mut Vec<u8>,
    link_process_channels: &mut Vec<crossbeam_channel::Sender<CdpTuple<T>>>,
    validator_thread_handles: &mut Vec<std::thread::JoinHandle<()>>,
    config: std::sync::Arc<impl util::lib::Config + 'static>,
    stats_sender_channel: std::sync::mpsc::Sender<crate::stats::stats_controller::StatType>,
) {
    for (rdh, data, mem_pos) in cdp_chunk.into_iter() {
        if let Some(link_index) = links.iter().position(|&x| x == rdh.link_id()) {
            link_process_channels
                .get(link_index)
                .unwrap()
                .send((rdh, data, mem_pos))
                .unwrap();
        } else {
            links.push(rdh.link_id());
            let (send_channel, recv_channel) =
                crossbeam_channel::bounded(crate::CHANNEL_CDP_CAPACITY);
            link_process_channels.push(send_channel);

            validator_thread_handles.push(
                std::thread::Builder::new()
                    .name(format!("Link {} Validator", rdh.link_id()))
                    .spawn({
                        let config = config.clone();
                        let stats_sender_channel = stats_sender_channel.clone();
                        let mut link_validator =
                            LinkValidator::new(&*config, stats_sender_channel, recv_channel);
                        move || {
                            link_validator.run();
                        }
                    })
                    .expect("Failed to spawn link validator thread"),
            );
            link_process_channels
                .last()
                .unwrap()
                .send((rdh, data, mem_pos))
                .unwrap();
        }
    }
}
