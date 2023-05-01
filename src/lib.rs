#![warn(missing_docs)]
//! fast Protocol Analysis Scanner Tool for ALICE (fastPASTA), for reading and checking raw binary data from ALICE detectors
//!
//! # Usage
//!
//! ## Reading data from file and performing checks
//! ```shell
//! # Enable all generic checks: `sanity` (stateless) AND `running` (stateful)
//! $ fastpasta <input_file> check all
//!
//! # Same as above but only enable `sanity` checks, and only check data from link 0
//! $ fastpasta <input_file>  check sanity -f 0
//!```
//! ## Enable all `sanity` and `running` checks and include checks applicable to `ITS` only
//! ```shell
//! $ fastpasta <input_file> check all ITS
//! ```
//! ## Filter link 3 and check `sanity` include sanity checks specific to ITS
//! ```shell
//! # target `its` is case-insensitive
//! $ fastpasta <input_file> -f 3 check sanity its
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
use input::{bufreader_wrapper::BufferedReaderWrapper, input_scanner::InputScanner};
use stats::lib::{self, StatType, SystemId};
use util::{
    config::Cfg,
    lib::{Checks, DataOutputMode, InputOutput, Util, Views},
};
use validators::{its::its_payload_fsm_cont::ItsPayloadFsmContinuous, lib::ValidatorDispatcher};
use words::{
    lib::RdhSubWord,
    rdh::Rdh0,
    rdh_cru::{RdhCRU, V6, V7},
};

use crate::util::lib::Filter;

pub mod input;
pub mod stats;
pub mod util;
pub mod validators;
pub mod view;
pub mod words;
pub mod write;

/// Does the initial setup for input data processing
pub fn init_processing(
    mut reader: Box<dyn BufferedReaderWrapper>,
    stat_send_channel: std::sync::mpsc::Sender<StatType>,
    thread_stopper: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> std::process::ExitCode {
    // Determine RDH version
    let rdh0 = Rdh0::load(&mut reader).expect("Failed to read first RDH0");
    let rdh_version = rdh0.header_id;

    // Send RDH version to stats thread
    stat_send_channel
        .send(StatType::RdhVersion(rdh_version))
        .unwrap();

    // Create input scanner from the already read RDH0 (to avoid seeking back and reading it twice, which would also break with stdin piping)
    let loader = InputScanner::new_from_rdh0(reader, stat_send_channel.clone(), rdh0);

    // Choose the rest of the execution based on the RDH version
    // Necessary to prevent heap allocation and allow static dispatch as the type cannot be known at compile time
    match rdh_version {
        6 => match process::<RdhCRU<V6>>(loader, stat_send_channel.clone(), thread_stopper) {
            Ok(_) => exit_success(),
            Err(e) => exit_fatal(stat_send_channel, e.to_string(), 2),
        },
        7 => match process::<RdhCRU<V7>>(loader, stat_send_channel.clone(), thread_stopper) {
            Ok(_) => exit_success(),
            Err(e) => exit_fatal(stat_send_channel, e.to_string(), 2),
        },
        _ => exit_fatal(
            stat_send_channel,
            format!("Unknown RDH version: {rdh_version}"),
            3,
        ),
    }
}

/// Entry point for scanning the input and delegating to checkers, view generators and/or writers depending on [Config]
///
/// Follows these steps:
/// 1. Setup reading (`file` or `stdin`) using [input::lib::spawn_reader].
/// 2. Depending on [Config] do one of:
///     - Validate data by dispatching it to validators with [validators::lib::ValidatorDispatcher].
///     - Generate views of data with [view::lib::generate_view].
///     - Write data to `file` or `stdout` with [write::lib::spawn_writer].
pub fn process<T: words::lib::RDH + 'static>(
    loader: InputScanner<impl BufferedReaderWrapper + ?Sized + std::marker::Send + 'static>,
    send_stats_ch: std::sync::mpsc::Sender<StatType>,
    thread_stopper: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> std::io::Result<()> {
    // 1. Launch reader thread to read data from file or stdin
    let (reader_handle, reader_rcv_channel): (
        std::thread::JoinHandle<()>,
        crossbeam_channel::Receiver<input::data_wrapper::CdpChunk<T>>,
    ) = input::lib::spawn_reader(thread_stopper.clone(), loader);

    // 2. Launch analysis thread if an analysis action is set (view or check)
    let analysis_handle = if Cfg::global().check().is_some() || Cfg::global().view().is_some() {
        debug_assert!(
            Cfg::global().output_mode() == util::lib::DataOutputMode::None
                || Cfg::global().filter_link().is_some()
        );
        let handle = spawn_analysis(
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
        Cfg::global().check(),
        Cfg::global().view(),
        Cfg::global().filter_link(),
        Cfg::global().output_mode(),
    ) {
        (None, None, Some(_), output_mode) if output_mode != DataOutputMode::None => {
            Some(write::lib::spawn_writer(thread_stopper, reader_rcv_channel))
        }
        (Some(_), None, _, output_mode) | (None, Some(_), _, output_mode)
            if output_mode != DataOutputMode::None =>
        {
            log::warn!(
                "Config: Output destination set when checks or views are also set -> output will be ignored!"
            );
            None
        }
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

/// Analysis thread that performs checks with [validators::lib::check_cdp_chunk] or generate views with [view::lib::generate_view].
fn spawn_analysis<T: words::lib::RDH + 'static>(
    stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    stats_sender_channel: std::sync::mpsc::Sender<StatType>,
    data_channel: Receiver<input::data_wrapper::CdpChunk<T>>,
) -> std::thread::JoinHandle<()> {
    let analysis_thread = std::thread::Builder::new().name("Analysis".to_string());
    let mut system_id: Option<SystemId> = None; // System ID is only set once
    analysis_thread
        .spawn({
            move || {
                // Setup for check case
                let mut validator_dispatcher =
                    ValidatorDispatcher::new(stats_sender_channel.clone());
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
                            stats_sender_channel.send(StatType::HBFsSeen(1)).unwrap();
                        }
                        if let Err(e) = lib::collect_system_specific_stats(
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
                    if Cfg::global().check().is_some() {
                        validator_dispatcher.dispatch_cdp_chunk(cdp_chunk);
                    } else if Cfg::global().view().is_some() {
                        if let Err(e) = view::lib::generate_view(
                            Cfg::global().view().unwrap(),
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

/// Start the [stderrlog] instance, and immediately use it to log the configured [DataOutputMode].
pub fn init_error_logger() {
    stderrlog::new()
        .module(module_path!())
        .verbosity(Cfg::global().verbosity() as usize)
        .init()
        .expect("Failed to initialize logger");
    match Cfg::global().output_mode() {
        util::lib::DataOutputMode::Stdout => log::trace!("Data ouput set to stdout"),
        util::lib::DataOutputMode::File => log::trace!("Data ouput set to file"),
        util::lib::DataOutputMode::None => {
            log::trace!("Data ouput set to suppressed")
        }
    }
}

/// Exit with [std::process::ExitCode] `SUCCESS`.
fn exit_success() -> std::process::ExitCode {
    log::info!("Exit successful");
    std::process::ExitCode::SUCCESS
}

fn exit_fatal(
    stat_send_channel: std::sync::mpsc::Sender<StatType>,
    error_string: String,
    exit_code: u8,
) -> std::process::ExitCode {
    stat_send_channel
        .send(StatType::Fatal(error_string))
        .unwrap();
    std::process::ExitCode::from(exit_code)
}
