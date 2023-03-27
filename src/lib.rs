//! fast Protocol Analysis Scanner Tool for ALICE (fastPASTA), for reading and checking raw binary data from ALICE detectors
//!
//! # Usage
//!
//! ## Reading data from file and performing checks on RDHs
//!
//! ```shell
//! $ fastpasta <input_file> check all
//! ```
//!
//! ## Reading data from stdin and performing all checks that applies to ITS
//!
//! ```shell
//! $ cat <input_file> | fastpasta check all ITS
//! ```
//!
//! ## reading data from one file, filtering by link 3 and and writing to another
//!
//! ```bash
//! $ fastpasta <input_file> --filter-link 3 -o <output_file>
//! ```
//!
//! ## Reading from stdin and filtering by link ID and writing to stdout
//! Writing to stdout is implicit when no checks or views are specified
//! ```bash
//! $ fastpasta <input_file> --filter-link 3
//! ```
//!
//! ## Reading from file and printing a view of RDHs
//!
//! ```bash
//! $ fastpasta <input_file> view rdh
//! ```

use crossbeam_channel::Receiver;
use util::lib::{Config, DataOutputMode};

pub mod data_write;
pub mod input;
pub mod stats;
pub mod util;
pub mod validators;
mod view;
pub mod words;

/// Capacity of the channel (FIFO) to Link Validator threads in terms of CDPs (RDH, Payload, Memory position)
///
///
/// Larger capacity means less overhead, but more memory usage
/// Too small capacity will cause the producer thread to block
const CHANNEL_CDP_CAPACITY: usize = 100;

// 1. Setup reading (file or stdin)
// 2. Do checks on read data
// 3. Write data out (file or stdout)
pub fn process<T: words::lib::RDH + 'static>(
    config: std::sync::Arc<impl Config + 'static>,
    loader: input::input_scanner::InputScanner<
        impl input::bufreader_wrapper::BufferedReaderWrapper + ?Sized + std::marker::Send + 'static,
    >,
    send_stats_ch: std::sync::mpsc::Sender<stats::stats_controller::StatType>,
    thread_stopper: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> std::io::Result<()> {
    // 1. Launch reader thread to read data from file or stdin
    let (reader_handle, reader_rcv_channel): (
        std::thread::JoinHandle<()>,
        crossbeam_channel::Receiver<input::data_wrapper::CdpChunk<T>>,
    ) = input::lib::spawn_reader(thread_stopper.clone(), loader);

    // 2. Launch analysis thread if an analysis action is set (view or check)
    let analysis_handle = if config.check().is_some() || config.view().is_some() {
        debug_assert!(
            config.output_mode() == util::lib::DataOutputMode::None
                || config.filter_link().is_some()
        );
        let handle = spawn_analysis(
            config.clone(),
            thread_stopper.clone(),
            send_stats_ch,
            reader_rcv_channel.clone(),
        );
        Some(handle)
    } else {
        None
    };

    // 3. Write data out only in the case where no analysis is performed and a filter link is set
    let output_handle: Option<std::thread::JoinHandle<()>> = match (
        config.check(),
        config.view(),
        config.filter_link(),
        config.output_mode(),
    ) {
        (None, None, Some(_), output_mode) if output_mode != DataOutputMode::None => Some(
            data_write::lib::spawn_writer(config.clone(), thread_stopper, reader_rcv_channel),
        ),
        _ => None,
    };

    reader_handle.join().expect("Error joining reader thread");

    if let Some(handle) = analysis_handle {
        if let Err(e) = handle.join() {
            log::error!("Analysis thread terminated early: {:#?}\n", e);
        }
    }
    if let Some(output) = output_handle {
        output.join().expect("Could not join writer thread");
    }
    Ok(())
}

fn spawn_analysis<T: words::lib::RDH + 'static>(
    config: std::sync::Arc<impl Config + 'static>,
    stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    stats_sender_channel: std::sync::mpsc::Sender<stats::stats_controller::StatType>,
    data_channel: Receiver<input::data_wrapper::CdpChunk<T>>,
) -> std::thread::JoinHandle<()> {
    let analysis_thread = std::thread::Builder::new().name("Analysis".to_string());

    analysis_thread
        .spawn({
            move || {
                type CdpTuple<T> = (T, Vec<u8>, u64);
                let mut links: Vec<u8> = Vec::new();
                let mut link_process_channels: Vec<crossbeam_channel::Sender<CdpTuple<T>>> =
                    Vec::new();
                let mut validator_thread_handles: Vec<std::thread::JoinHandle<()>> = Vec::new();
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
                    cdp_chunk.rdh_slice().iter().for_each(|rdh| {
                        if rdh.stop_bit() == 1 {
                            stats_sender_channel
                                .send(stats::stats_controller::StatType::HBFsSeen(1))
                                .unwrap();
                        }
                        let layer = words::lib::layer_from_feeid(rdh.fee_id());
                        let stave = words::lib::stave_number_from_feeid(rdh.fee_id());
                        stats_sender_channel
                            .send(stats::stats_controller::StatType::LayerStaveSeen {
                                layer,
                                stave,
                            })
                            .unwrap();
                        stats_sender_channel
                            .send(stats::stats_controller::StatType::DataFormat(
                                rdh.data_format(),
                            ))
                            .unwrap();
                    });

                    // Do checks or view
                    if config.check().is_some() {
                        cdp_chunk.into_iter().for_each(|(rdh, data, mem_pos)| {
                            if let Some(link_index) = links.iter().position(|&x| x == rdh.link_id())
                            {
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
                                use crate::validators::link_validator::LinkValidator;
                                validator_thread_handles.push(
                                    std::thread::Builder::new()
                                        .name(format!("Link {} Validator", rdh.link_id()))
                                        .spawn({
                                            let config = config.clone();
                                            let stats_sender_channel = stats_sender_channel.clone();
                                            let mut link_validator = LinkValidator::new(
                                                &*config,
                                                stats_sender_channel,
                                                recv_channel,
                                            );
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
                        });
                    } else if config.view().is_some() {
                        view::lib::generate_view(
                            config.view().unwrap(),
                            cdp_chunk,
                            &stats_sender_channel,
                        );
                    }
                }
                // Stop all threads
                link_process_channels.clear();
                validator_thread_handles.into_iter().for_each(|handle| {
                    handle.join().expect("Failed to join a validator thread");
                });
            }
        })
        .expect("Failed to spawn checker thread")
}
