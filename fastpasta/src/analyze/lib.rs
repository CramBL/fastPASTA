//! Contains the [spawn_analysis] function that spawns the analysis thread for either data validation or view generation.
use super::validators::validator_dispatcher::ValidatorDispatcher;
use crate::config::lib::Config;
use crate::stats;
use crate::stats::StatType;
use crate::stats::SystemId;
use alice_protocol_reader::cdp_arr::CdpArr;
use alice_protocol_reader::prelude::*;
use crossbeam_channel::Receiver;

/// Analysis thread that performs checks with the [super::validators] module or generate views with the [super::view::lib::generate_view] function.
pub fn spawn_analysis<T: RDH + 'static, const CAP: usize>(
    config: &'static impl Config,
    stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    stats_send: flume::Sender<StatType>,
    data_recv: Receiver<CdpArr<T, CAP>>,
) -> std::thread::JoinHandle<()> {
    let analysis_thread = std::thread::Builder::new().name("Analysis".to_string());
    let mut system_id: Option<SystemId> = None; // System ID is only set once
    analysis_thread
        .spawn({
            move || {
                // Setup for check case
                let mut validator_dispatcher = ValidatorDispatcher::new(config, stats_send.clone());
                // Start analysis
                while !stop_flag.load(std::sync::atomic::Ordering::SeqCst) {
                    // Receive chunk from reader
                    let cdp_chunk = match data_recv.recv() {
                        Ok(cdp) => cdp,
                        Err(e) => {
                            debug_assert_eq!(e, crossbeam_channel::RecvError);
                            break;
                        }
                    };

                    // Collect global stats
                    // Send HBF seen if stop bit is 1
                    for rdh in cdp_chunk.rdh_slice().iter() {
                        if rdh.stop_bit() == 1 {
                            stats_send.send(StatType::HBFSeen).unwrap();
                        }
                        stats_send
                            .send(StatType::TriggerType(rdh.trigger_type()))
                            .unwrap();
                        if let Err(e) =
                            stats::collect_system_specific_stats(rdh, &mut system_id, &stats_send)
                        {
                            // Send error and break, stop processing
                            stats_send.send(StatType::Fatal(e.into())).unwrap();
                            break; // Fatal error
                        }
                    }

                    // Do checks or view
                    if config.check().is_some() {
                        validator_dispatcher.dispatch_cdp_chunk(cdp_chunk);
                    } else if let Some(view) = config.view() {
                        if let Err(e) = super::view::lib::generate_view(view, cdp_chunk) {
                            stats_send
                                .send(StatType::Fatal(e.to_string().into()))
                                .expect("Couldn't send to Controller");
                        }
                    }
                }
                // Join all threads the dispatcher spawned
                validator_dispatcher.join();
            }
        })
        .expect("Failed to spawn checker thread")
}
