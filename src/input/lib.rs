//! This module contains mainly the [InputScanner] that reads the input data, and the [CdpChunk] data structure that wraps the data read from the input.
//! Additionally it contains a helper function [spawn_reader] that spawns a thread that reads input and sents it to a channel that is returned from the function.
//!
//! The [InputScanner] is a generic type that can be instantiated with any type that implements the [BufferedReaderWrapper] trait.
//! This trait is implemented for the [StdInReaderSeeker] and the [std::io::BufReader] types.
//! Allowing the [InputScanner] to read from both stdin and files, in a convenient and effecient way.
//!
//! The [CdpChunk] is a wrapper for the data read from the input, it contains the data and the memory address of the first byte of the data.

use super::prelude::RDH;
use super::{
    bufreader_wrapper::BufferedReaderWrapper,
    data_wrapper::CdpChunk,
    input_scanner::{InputScanner, ScanCDP},
    stdin_reader::StdInReaderSeeker,
};
use crate::stats;
use crate::stats::StatType;
use crate::stats::SystemId;
use crossbeam_channel::Receiver;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{io::IsTerminal, path::PathBuf};

/// Depth of the FIFO where the CDP chunks inserted as they are read
const CHANNEL_CDP_CHUNK_CAPACITY: usize = 100;
const READER_BUFFER_SIZE: usize = 1024 * 50; // 50KB

/// Initializes the reader based on the input mode (file or stdin) and returns it
///
/// The input mode is determined by the presence of the input file path in the config
#[inline]
pub fn init_reader(
    input_file: &Option<PathBuf>,
) -> Result<Box<dyn BufferedReaderWrapper>, std::io::Error> {
    if let Some(path) = input_file {
        log::trace!("Reading from file: {:?}", &path);
        let f = std::fs::OpenOptions::new().read(true).open(path)?;
        Ok(Box::new(std::io::BufReader::with_capacity(
            READER_BUFFER_SIZE,
            f,
        )))
    } else {
        log::trace!("Reading from stdin");
        if !std::io::stdin().is_terminal() {
            Ok(Box::new(StdInReaderSeeker {
                reader: std::io::stdin(),
            }))
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "stdin not redirected!",
            ))
        }
    }
}

/// Spawns a reader thread that reads CDPs from the input and sends them to a producer channel
///
/// Returns the thread handle and the receiver channel
pub fn spawn_reader<T: RDH + 'static>(
    stop_flag: std::sync::Arc<AtomicBool>,
    input_scanner: InputScanner<impl BufferedReaderWrapper + ?Sized + std::marker::Send + 'static>,
    stats_sender_channel: flume::Sender<StatType>,
) -> (std::thread::JoinHandle<()>, Receiver<CdpChunk<T>>) {
    let reader_thread = std::thread::Builder::new().name("Reader".to_string());
    let (send_channel, rcv_channel) = crossbeam_channel::bounded(CHANNEL_CDP_CHUNK_CAPACITY);
    let mut local_stop_on_non_full_chunk = false;
    let mut system_id: Option<SystemId> = None; // System ID is only set once
    const CDP_CHUNK_SIZE: usize = 100;
    let thread_handle = reader_thread
        .spawn({
            move || {
                let mut input_scanner = input_scanner;

                // Automatically extracts link to filter if one is supplied
                while !stop_flag.load(Ordering::SeqCst) && !local_stop_on_non_full_chunk {
                    let cdps = match get_chunk::<T>(&mut input_scanner, CDP_CHUNK_SIZE) {
                        Ok(cdp) => {
                            if cdp.len() < CDP_CHUNK_SIZE {
                                local_stop_on_non_full_chunk = true; // Stop on non-full chunk, could be InvalidData
                                log::trace!("Stopping reader thread on non-full chunk");
                            }
                            cdp
                        }
                        Err(e) => {
                            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                                log::trace!("Stopping reader thread on EOF");
                                break;
                            } else {
                                log::error!("Unexpected Error reading CDP chunks: {e}");
                                break;
                            }
                        }
                    };

                    // Collect global stats
                    // Send HBF seen if stop bit is 1
                    for rdh in cdps.rdh_slice().iter() {
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

                    // Send a chunk to the checker
                    if let Err(e) = send_channel.send(cdps) {
                        if !stop_flag.load(Ordering::SeqCst) {
                            log::trace!("Unexpected error while sending data to checker: {e}");
                            break;
                        }
                    }
                }
            }
        })
        .expect("Failed to spawn reader thread");
    (thread_handle, rcv_channel)
}

/// Attempts to fill a CDP chunk with as many CDPs as possible (up to the chunk size) and returns it
///
/// If an error occurs after one or more CDPs have been read, the CDP chunk is returned with the CDPs read so far
/// If the error occurs before any CDPs have been read, the error is returned
#[inline]
fn get_chunk<T: RDH>(
    file_scanner: &mut InputScanner<impl BufferedReaderWrapper + ?Sized>,
    chunk_size_cdps: usize,
) -> Result<CdpChunk<T>, std::io::Error> {
    let mut cdp_chunk = CdpChunk::new();

    for _ in 0..chunk_size_cdps {
        let cdp_tuple = match file_scanner.load_cdp() {
            Ok(cdp) => cdp,
            Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
                log::trace!("Invalid data found, returning all CDPs found so far");
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                log::info!("EOF reached! ");
                break;
            }
            Err(e) => return Err(e),
        };
        cdp_chunk.push(cdp_tuple.0, cdp_tuple.1, cdp_tuple.2);
    }

    if cdp_chunk.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "No CDPs found",
        ));
    }

    Ok(cdp_chunk)
}
