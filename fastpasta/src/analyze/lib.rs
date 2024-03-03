//! Contains the [spawn_analysis] function that spawns the analysis thread for either data validation or view generation.
use super::validators::validator_dispatcher::ValidatorDispatcher;
use crate::util::*;

/// Analysis thread that performs checks with the [super::validators] module or generate views with the [super::view::lib::generate_view] function.
pub fn spawn_analysis<T: RDH + 'static, const CAP: usize>(
    config: &'static impl Config,
    stop_flag: Arc<AtomicBool>,
    stats_send: flume::Sender<StatType>,
    data_recv: crossbeam_channel::Receiver<CdpArray<T, CAP>>,
) -> Result<JoinHandle<()>, io::Error> {
    let analysis_thread = thread::Builder::new().name("Analysis".to_string());
    let mut system_id: Option<SystemId> = None; // System ID is only set once
    analysis_thread.spawn({
        move || {
            // Setup for check case
            let mut validator_dispatcher = ValidatorDispatcher::new(config, stats_send.clone());
            // Start analysis
            while !stop_flag.load(Ordering::SeqCst) {
                // Receive batch from reader
                let cdp_batch = match data_recv.recv() {
                    Ok(cdp) => cdp,
                    Err(e) => {
                        debug_assert_eq!(e, crossbeam_channel::RecvError);
                        break;
                    }
                };

                // Collect global stats
                // Send HBF seen if stop bit is 1
                for rdh in cdp_batch.rdh_slice().iter() {
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
                    validator_dispatcher.dispatch_cdp_batch(cdp_batch);
                } else if let Some(view) = config.view() {
                    if let Err(e) = view::lib::generate_view(view, &cdp_batch) {
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
}
