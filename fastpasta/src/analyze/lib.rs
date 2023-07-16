//! Contains the [spawn_analysis] function that spawns the analysis thread for either data validation or view generation.
use super::validators::its::its_payload_fsm_cont::ItsPayloadFsmContinuous;
use super::validators::lib::ValidatorDispatcher;
use crate::config::lib::Config;
use crate::input;
use crate::stats;
use crate::stats::StatType;
use crate::stats::SystemId;
use crossbeam_channel::Receiver;
use input::prelude::*;

/// Analysis thread that performs checks with the [super::validators] module or generate views with the [super::view::lib::generate_view] function.
pub fn spawn_analysis<T: RDH + 'static>(
    config: &'static impl Config,
    stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    stats_sender_channel: flume::Sender<StatType>,
    data_channel: Receiver<CdpChunk<T>>,
) -> std::thread::JoinHandle<()> {
    let analysis_thread = std::thread::Builder::new().name("Analysis".to_string());
    let mut system_id: Option<SystemId> = None; // System ID is only set once
    analysis_thread
        .spawn({
            move || {
                // Setup for check case
                let mut validator_dispatcher =
                    ValidatorDispatcher::new(config, stats_sender_channel.clone());
                // Setup for view case
                let mut its_payload_fsm_cont = ItsPayloadFsmContinuous::default();
                // Start analysis
                while !stop_flag.load(std::sync::atomic::Ordering::SeqCst) {
                    // Receive chunk from reader
                    let cdp_chunk = match data_channel.recv() {
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
                            stats_sender_channel.send(StatType::HBFSeen).unwrap();
                        }
                        stats_sender_channel
                            .send(StatType::TriggerType(rdh.trigger_type()))
                            .unwrap();
                        if let Err(e) = stats::collect_system_specific_stats(
                            rdh,
                            &mut system_id,
                            &stats_sender_channel,
                        ) {
                            // Send error and break, stop processing
                            stats_sender_channel.send(StatType::Fatal(e)).unwrap();
                            break; // Fatal error
                        }
                    }

                    // Do checks or view
                    if config.check().is_some() {
                        validator_dispatcher.dispatch_cdp_chunk(cdp_chunk);
                    } else if config.view().is_some() {
                        if let Err(e) = super::view::lib::generate_view(
                            config.view().unwrap(),
                            cdp_chunk,
                            &stats_sender_channel,
                            &mut its_payload_fsm_cont,
                        ) {
                            stats_sender_channel
                                .send(StatType::Fatal(e.to_string()))
                                .expect("Couldn't send to StatsController");
                        }
                    }
                }
                // Join all threads the dispatcher spawned
                validator_dispatcher.join();
            }
        })
        .expect("Failed to spawn checker thread")
}
