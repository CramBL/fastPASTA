//! fast Protocol Analysis Scanner Tool for ALICE (fastPASTA), for reading and checking raw binary data from ALICE detectors
//!
//! # Usage
//!
//! ## Reading data from file and performing checks
//!
//! ```shell
//! $ fastpasta <input_file> check all
//! ```
//!
//! ## Reading data from stdin and performing all checks on only RDH
//!
//! ```shell
//! $ cat <input_file> | fastpasta check all rdh
//! ```
//!
//! ## reading data from one file and writing to another
//!
//! ```bash
//! $ fastpasta <input_file> -o <output_file>
//! ```
//!
//! ## Reading from stdin and filtering by link ID and writing to stdout
//! Writing to stdout is implicit when no checks or views are specified
//! ```bash
//! $ fastpasta <input_file> --filter-link 1
//! ```
//!
//! ## Reading from file and printing a view of RDHs
//!
//! ```bash
//! $ fastpasta <input_file> view rdh
//! ```

use crossbeam_channel::Receiver;
use util::lib::Config;

pub mod data_write;
pub mod input;
pub mod stats;
pub mod util;
pub mod validators;
pub mod words;

// Larger capacity means less overhead, but more memory usage
// Too small capacity will cause the producer thread to block
// Too large capacity will cause down stream consumers to block
pub const CHANNEL_CDP_CAPACITY: usize = 100;

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
    // 1. Read data from file or stdin
    let (reader_handle, reader_rcv_channel): (
        std::thread::JoinHandle<()>,
        crossbeam_channel::Receiver<input::data_wrapper::CdpChunk<T>>,
    ) = input::lib::spawn_reader(thread_stopper.clone(), loader);

    // 2. Launch action thread if an action is set (view or check)
    let action_handle = if config.check().is_some() || config.view().is_some() {
        debug_assert!(
            config.output_mode() == util::lib::DataOutputMode::None
                || config.filter_link().is_some()
        );
        let handle = spawn_action_thread(
            config.clone(),
            thread_stopper.clone(),
            send_stats_ch.clone(),
            reader_rcv_channel.clone(),
        );
        Some(handle)
    } else {
        None
    };

    // let (validator_handle, checker_rcv_channel) = validators::lib::spawn_validator::<T>(
    //     config.clone(),
    //     thread_stopper.clone(),
    //     send_stats_ch.clone(),
    //     reader_rcv_channel.clone(),
    // );

    // 3. Write data out
    let output_handle: Option<std::thread::JoinHandle<()>> = match config.output_mode() {
        util::lib::DataOutputMode::None => None,
        _ => Some(data_write::lib::spawn_writer(
            config.clone(),
            thread_stopper,
            reader_rcv_channel,
        )),
    };

    // // 3b. Print a view
    // Some(spawn_view::<T>(
    //     config,
    //     thread_stopper,
    //     reader_rcv_channel,
    //     send_stats_ch,
    // ))

    reader_handle.join().expect("Error joining reader thread");

    if let Some(handle) = action_handle {
        if let Err(e) = handle.join() {
            log::error!("Action thread terminated early: {:#?}\n", e);
        }
    }
    if let Some(output) = output_handle {
        output.join().expect("Could not join writer thread");
    }
    Ok(())
}

fn spawn_view<T: words::lib::RDH + 'static>(
    config: std::sync::Arc<impl Config + 'static>,
    stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    data_channel: crossbeam_channel::Receiver<input::data_wrapper::CdpChunk<T>>,
    send_stats_ch: std::sync::mpsc::Sender<stats::stats_controller::StatType>,
) -> std::thread::JoinHandle<()> {
    let view_thread = std::thread::Builder::new().name("View".to_string());
    view_thread
        .spawn({
            move || loop {
                // Receive chunk from checker
                let cdps = match data_channel.recv() {
                    Ok(cdp) => cdp,
                    Err(e) => {
                        debug_assert_eq!(e, crossbeam_channel::RecvError);
                        break;
                    }
                };
                if stop_flag.load(std::sync::atomic::Ordering::SeqCst) {
                    log::trace!("Stopping view thread");
                    break;
                }
                // Print a view
                if let Some(view) = config.view() {
                    match view {
                        util::config::View::Rdh => {
                            let header_text =
                                words::rdh_cru::RdhCRU::<T>::rdh_header_text_with_indent_to_string(
                                    16,
                                );
                            let mut stdio_lock = std::io::stdout().lock();
                            use std::io::Write;
                            if let Err(e) = writeln!(stdio_lock, "             {header_text}") {
                                send_stats_ch
                                    .send(stats::stats_controller::StatType::Fatal(format!(
                                        "Error while printing RDH header: {e}"
                                    )))
                                    .unwrap();
                            }
                            for (rdh, _, mem_pos) in &cdps {
                                if let Err(e) = writeln!(stdio_lock, "{mem_pos:>8X}:{rdh}") {
                                    send_stats_ch
                                        .send(stats::stats_controller::StatType::Fatal(format!(
                                            "Error while printing RDH header: {e}"
                                        )))
                                        .unwrap();
                                }
                            }
                        }
                    }
                }
            }
        })
        .expect("Failed to spawn view thread")
}

fn spawn_action_thread<T: words::lib::RDH + 'static>(
    config: std::sync::Arc<impl Config + 'static>,
    stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    stats_sender_channel: std::sync::mpsc::Sender<stats::stats_controller::StatType>,
    data_channel: Receiver<input::data_wrapper::CdpChunk<T>>,
) -> std::thread::JoinHandle<()> {
    let action_thread = std::thread::Builder::new().name("Action".to_string());

    let action_handle = action_thread
        .spawn({
            let config = config.clone();
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

                    // Do action
                    if config.check().is_some() {
                        cdp_chunk.into_iter().for_each(|(rdh, data, mem_pos)| {
                            if let Some(link_index) = links.iter().position(|&x| x == rdh.link_id())
                            {
                                //log::info!("Link {} already has a validator", rdh.link_id());
                                link_process_channels
                                    .get(link_index)
                                    .unwrap()
                                    .send((rdh, data, mem_pos))
                                    .unwrap();
                            } else {
                                //log::info!("Link {} has no validator, creating one", rdh.link_id());
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
                                            move || {
                                                let mut link_validator = LinkValidator::new(
                                                    &*config.clone(),
                                                    stats_sender_channel,
                                                    recv_channel,
                                                );
                                                link_validator.run();
                                            }
                                        })
                                        .expect("Failed to spawn link validator thread"),
                                );
                            }
                        });
                    }
                }
                // Stop all threads
                validator_thread_handles.into_iter().for_each(|handle| {
                    handle.join().expect("Failed to join a validator thread");
                });
            }
        })
        .expect("Failed to spawn checker thread");

    action_handle
}
