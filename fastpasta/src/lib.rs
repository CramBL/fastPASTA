#![warn(unused_extern_crates)]
#![warn(missing_docs)]
#![warn(missing_copy_implementations)]
// Readability lints
#![warn(
    clippy::option_filter_map,
    clippy::manual_filter_map,
    clippy::if_not_else,
    clippy::nonminimal_bool,
    clippy::single_match_else,
    clippy::range_plus_one,
    clippy::int_plus_one,
    clippy::needless_range_loop,
    clippy::needless_continue,
    clippy::shadow_same,
    clippy::shadow_unrelated
)]
// Performance lints
#![warn(variant_size_differences)]
#![warn(
    clippy::needless_pass_by_value,
    clippy::unnecessary_wraps,
    clippy::mutex_integer,
    clippy::mem_forget,
    clippy::maybe_infinite_iter
)]
// Safety lints
#![warn(unused_results)]
#![warn(unused_import_braces)]
#![warn(trivial_casts, trivial_numeric_casts)]
// Unhandled results (allow unwrap and expect as there are many cases where the unwrap is totally safe)
#![warn(clippy::map_unwrap_or)]

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

use crate::util::*;
use analyze::validators::rdh::Rdh0Validator;

/// Write an error message to stderr.
/// All error messages should be written through this function to ensure consistency.
#[inline]
pub fn display_error(err_msg: &str) {
    log::error!("{}", owo_colors::OwoColorize::red(&err_msg));
}

pub mod analyze;
pub mod config;
pub mod controller;
pub mod init;
pub mod stats;
pub mod util;
pub mod words;
pub mod write;

/// Does the initial setup for input data processing
#[allow(clippy::needless_pass_by_value)] // We need to pass the reader by value to avoid lifetime issues (thread just spins) unless user drops the sender after calling which is not intuitive
pub fn init_processing(
    config: &'static impl Config,
    mut reader: Box<dyn BufferedReaderWrapper>,
    stat_send: flume::Sender<StatType>,
    stop_flag: Arc<atomic::AtomicBool>,
) -> io::Result<()> {
    // Load the first few bytes that should contain RDH0 and do a basic sanity check before continuing.
    // Early exit if the check fails.
    let rdh0 = Rdh0::load(&mut reader).expect("Failed to read first RDH0");
    if let Err(e) = Rdh0Validator::default().sanity_check(&rdh0) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Initial RDH0 deserialization failed sanity check: {e}"),
        ));
    }
    // Determine RDH version
    let rdh_version = rdh0.header_id;

    // Send RDH version to stats thread
    stat_send.send(StatType::RdhVersion(rdh_version)).unwrap();

    // Create a receiver/sender channel for the stats that the InputScanner sends.
    let (input_stats_send, input_stats_recv): (
        flume::Sender<InputStatType>,
        flume::Receiver<InputStatType>,
    ) = flume::unbounded();
    // Create input scanner from the already read RDH0 (to avoid seeking back and reading it twice, which would also break with stdin piping)
    let loader = InputScanner::new_from_rdh0(config, reader, Some(input_stats_send), rdh0);

    // Choose the rest of the execution based on the RDH version
    // Necessary to prevent heap allocation and allow static dispatch as the type cannot be known at compile time
    match rdh_version {
        // Attempt to parse RDHs with version field in the range 3-100
        // Upper limit is 100 and not just max of u8 (255) because:
        //      1. Unlikely there will ever be an RDH version higher than that
        //      2. High values decoded from this field (especially 255) is typically a sign that the data is not actually ALICE data so early exit is preferred
        3..=100 => {
            match process::<RdhCru, 100>(
                config,
                loader,
                Some(&input_stats_recv),
                &stat_send,
                stop_flag,
            ) {
                Ok(_) => Ok(()),
                Err(e) => {
                    stat_send
                        .send(StatType::Fatal(e.to_string().into()))
                        .unwrap();
                    Err(e)
                }
            }
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unknown RDH version: {rdh_version}"),
        )),
    }
}

/// Entry point for scanning the input and delegating to checkers, view generators and/or writers depending on [Config]
///
/// Follows these steps:
/// 1. Setup reading (`file` or `stdin`) using [alice_protocol_reader::spawn_reader].
/// 2. Depending on [Config] do one of:
///     - Validate data by dispatching it to validators with [ValidatorDispatcher][crate::analyze::validators::validator_dispatcher::ValidatorDispatcher].
///     - Generate views of data with [analyze::view::lib::generate_view].
///     - Write data to `file` or `stdout` with [write::lib::spawn_writer].
pub fn process<T: RDH + 'static, const CAP: usize>(
    config: &'static impl Config,
    loader: InputScanner<impl BufferedReaderWrapper + ?Sized + marker::Send + 'static>,
    input_stats_recv: Option<&flume::Receiver<InputStatType>>,
    stats_send: &flume::Sender<StatType>,
    stop_flag: Arc<atomic::AtomicBool>,
) -> io::Result<()> {
    // 1. Launch reader thread to read data from file or stdin
    let (reader_handle, reader_data_recv): (
        thread::JoinHandle<()>,
        crossbeam_channel::Receiver<CdpArray<T, CAP>>,
    ) = alice_protocol_reader::spawn_reader(stop_flag.clone(), loader);

    // 2. Launch analysis thread if an analysis action is set (view or check)
    let analysis_handle = if config.check().is_some() || config.view().is_some() {
        debug_assert!(config.output_mode() == DataOutputMode::None || config.filter_enabled(),);
        let handle = analyze::lib::spawn_analysis(
            config,
            stop_flag.clone(),
            stats_send.clone(),
            reader_data_recv.clone(),
        )?;
        Some(handle)
    } else {
        None
    };

    // 3. Write data out only in the case where no analysis is performed and a filter link is set
    let output_handle: Option<thread::JoinHandle<()>> = match (
        config.check(),
        config.view(),
        config.filter_enabled(),
        config.output_mode(),
    ) {
        (None, None, true, output_mode) if output_mode != DataOutputMode::None => Some(
            write::lib::spawn_writer(config, stop_flag, reader_data_recv),
        ),

        (Some(_), None, _, output_mode) | (None, Some(_), _, output_mode)
            if output_mode != DataOutputMode::None =>
        {
            log::warn!(
                "Config: Output destination set when checks or views are also set -> output will be ignored!"
            );
            drop(reader_data_recv);
            None
        }
        _ => {
            drop(reader_data_recv);
            None
        }
    };

    // While loop breaks when an error is received from the channel, which means the channel is disconnected
    if let Some(input_stats_recv_chan) = input_stats_recv.as_ref() {
        forward_input_stats_to_stats_collector(input_stats_recv_chan, stats_send);
    }
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

// This is basically a "glue" function that takes the stats types that the reader sends
// handles the transformation needed to send them in the format the the stats collector expects
// and sends them
fn forward_input_stats_to_stats_collector(
    input_stats_recv: &flume::Receiver<InputStatType>,
    stats_send: &flume::Sender<StatType>,
) {
    while let Ok(input_stat) = input_stats_recv.recv() {
        match input_stat {
            InputStatType::LinksObserved(val) => {
                stats_send.send(StatType::LinksObserved(val)).unwrap()
            }
            InputStatType::FeeId(val) => stats_send.send(StatType::FeeId(val)).unwrap(),
            InputStatType::RDHSeen(val) => stats_send.send(StatType::RDHSeen(val)).unwrap(),
            InputStatType::PayloadSize(val) => stats_send.send(StatType::PayloadSize(val)).unwrap(),
            InputStatType::RDHFiltered(val) => stats_send.send(StatType::RDHFiltered(val)).unwrap(),
            InputStatType::RunTriggerType(val) => stats_send
                .send(StatType::RunTriggerType((
                    val,
                    crate::analyze::view::lib::trigger_type_string_from_int(val),
                )))
                .unwrap(),
            InputStatType::DataFormat(val) => stats_send.send(StatType::DataFormat(val)).unwrap(),
            InputStatType::SystemId(sys_id) => {
                match stats::SystemId::from_system_id(sys_id) {
                    Ok(id) => {
                        log::info!("{id} detected");
                        stats_send.send(StatType::SystemId(id)).unwrap()
                    }
                    Err(e) => {
                        log::error!("Failed to parse system ID: {e}");
                        stats_send
                            .send(StatType::Fatal("Failed to parse system ID".into()))
                            .unwrap();
                    }
                };
            }
            InputStatType::Error(e) => stats_send.send(StatType::Error(e)).unwrap(),
            InputStatType::Fatal(e) => stats_send.send(StatType::Fatal(e)).unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alice_protocol_reader::init_reader;
    use alice_protocol_reader::prelude::test_data::CORRECT_RDH_CRU_V7;
    use pretty_assertions::{assert_eq, assert_ne};

    static CFG_TEST_INIT_PROCESSING: OnceLock<MockConfig> = OnceLock::new();

    #[test]
    fn test_init_processing() {
        // Setup Mock Config
        let mut mock_config = MockConfig::new();
        // Set input file from one of the files used for regression testing
        mock_config.input_file = Some(PathBuf::from("../tests/test-data/10_rdh.raw"));

        CFG_TEST_INIT_PROCESSING.set(mock_config).unwrap();

        // Setup a reader
        let reader = init_reader(CFG_TEST_INIT_PROCESSING.get().unwrap().input_file()).unwrap();

        let (sender, receiver): (flume::Sender<StatType>, flume::Receiver<StatType>) =
            flume::unbounded();

        let stop_flag = Arc::new(AtomicBool::new(false));

        // Act
        init_processing(
            CFG_TEST_INIT_PROCESSING.get().unwrap(),
            reader,
            sender,
            stop_flag.clone(),
        )
        .unwrap();

        // Receive all messages
        let mut stats: Vec<StatType> = Vec::new();

        while let Ok(stat) = receiver.recv() {
            stats.push(stat);
        }

        // Assert
        let mut is_rdh_version_detected_7 = false;
        let mut how_many_rdh_seen = 0;

        // Print all stats
        for stat in stats {
            match stat {
                StatType::RdhVersion(7) => is_rdh_version_detected_7 = true,
                StatType::RDHSeen(val) => how_many_rdh_seen += val,
                StatType::Error(e) | StatType::Fatal(e) => {
                    panic!("Error or Fatal: {}", e)
                }
                _ => (),
            }
        }

        assert!(is_rdh_version_detected_7);
        assert_eq!(how_many_rdh_seen, 10);
        assert!(!stop_flag.load(Ordering::SeqCst));
    }

    static CFG_TEST_SPAWN_ANALYSIS: OnceLock<MockConfig> = OnceLock::new();

    #[test]
    fn test_spawn_analysis() {
        // Setup Mock Config, no checks or views to be done
        let mock_config = MockConfig::default();
        CFG_TEST_SPAWN_ANALYSIS.set(mock_config).unwrap();
        let (stat_sender, stat_receiver): (flume::Sender<StatType>, flume::Receiver<StatType>) =
            flume::unbounded();
        let (data_sender, data_receiver) = crossbeam_channel::unbounded();
        let stop_flag = Arc::new(AtomicBool::new(false));
        let mut cdp_batch: CdpArray<RdhCru, 1> = CdpArray::new();
        cdp_batch.push(CORRECT_RDH_CRU_V7, Vec::new(), 0);

        // Act
        let handle = analyze::lib::spawn_analysis(
            CFG_TEST_SPAWN_ANALYSIS.get().unwrap(),
            stop_flag.clone(),
            stat_sender,
            data_receiver,
        )
        .unwrap();
        data_sender.send(cdp_batch).unwrap();
        drop(data_sender);
        // Sleep to give the thread time to process the data
        thread::sleep(Duration::from_millis(100));
        stop_flag.store(true, Ordering::SeqCst);

        // Receive all messages
        let mut stats: Vec<StatType> = Vec::new();
        while let Ok(stat) = stat_receiver.recv() {
            stats.push(stat);
        }

        // Some stats should have been sent
        assert_ne!(
            stats.len(),
            0,
            "Expected some stats received, got: {stats:?}"
        );
        handle.join().unwrap();
    }
}
