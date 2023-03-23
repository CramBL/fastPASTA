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
    // 1. Read data from file
    let (reader_handle, reader_rcv_channel) =
        input::lib::spawn_reader(thread_stopper.clone(), loader);

    // 2. Do checks on a received chunk of data
    let (validator_handle, checker_rcv_channel) = validators::lib::spawn_validator::<T>(
        config.clone(),
        thread_stopper.clone(),
        send_stats_ch.clone(),
        reader_rcv_channel.clone(),
    );

    // 3. Write data out or Print a view
    let output_handle: Option<std::thread::JoinHandle<()>> = if config.view().is_none() {
        // 3a. Write data out
        match config.output_mode() {
            util::lib::DataOutputMode::None => None,
            _ => Some(data_write::lib::spawn_writer(
                config.clone(),
                thread_stopper,
                checker_rcv_channel.expect("Checker receiver channel not initialized"),
            )),
        }
    } else {
        // 3b. Print a view
        Some(spawn_view::<T>(
            config,
            thread_stopper,
            reader_rcv_channel,
            send_stats_ch,
        ))
    };

    reader_handle.join().expect("Error joining reader thread");
    if let Err(e) = validator_handle.join() {
        log::error!("Validator thread terminated early: {:#?}\n", e);
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
