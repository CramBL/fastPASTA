#![warn(missing_docs)]

//! This module contains mainly the [InputScanner] that reads the input data, and the [CdpChunk] data structure that wraps the data read from the input.
//! Additionally it contains a helper function [spawn_reader] that spawns a thread that reads input and sents it to a channel that is returned from the function.
//!
//! The [InputScanner] is a generic type that can be instantiated with any type that implements the [BufferedReaderWrapper] trait.
//! This trait is implemented for the [StdInReaderSeeker] and the [std::io::BufReader] types.
//! Allowing the [InputScanner] to read from both stdin and files, in a convenient and effecient way.
//!
//! The [CdpChunk] is a wrapper for the data read from the input, it contains the data and the memory address of the first byte of the data.

pub mod bufreader_wrapper;
pub mod config;
pub mod data_wrapper;
pub mod input_scanner;
pub mod mem_pos_tracker;
pub mod prelude;
pub mod rdh;
pub mod stdin_reader;

use crossbeam_channel::Receiver;
use prelude::{BufferedReaderWrapper, CdpChunk, InputScanner, ScanCDP, RDH};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{io::IsTerminal, path::PathBuf};
use stdin_reader::StdInReaderSeeker;

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
        let f = std::fs::OpenOptions::new().read(true).open(path)?;
        Ok(Box::new(std::io::BufReader::with_capacity(
            READER_BUFFER_SIZE,
            f,
        )))
    } else if !std::io::stdin().is_terminal() {
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

/// Spawns a reader thread that reads CDPs from the input and sends them to a producer channel
///
/// Returns the thread handle and the receiver channel
#[inline]
pub fn spawn_reader<T: RDH + 'static>(
    stop_flag: std::sync::Arc<AtomicBool>,
    input_scanner: InputScanner<impl BufferedReaderWrapper + ?Sized + std::marker::Send + 'static>,
) -> (std::thread::JoinHandle<()>, Receiver<CdpChunk<T>>) {
    let reader_thread = std::thread::Builder::new().name("Reader".to_string());
    let (send_channel, rcv_channel) = crossbeam_channel::bounded(CHANNEL_CDP_CHUNK_CAPACITY);
    let mut local_stop_on_non_full_chunk = false;
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
                            }
                            cdp
                        }
                        Err(_) => {
                            break;
                        }
                    };

                    // Send a chunk to the checker
                    if send_channel.send(cdps).is_err() {
                        break;
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
#[inline(always)]
fn get_chunk<T: RDH>(
    file_scanner: &mut InputScanner<impl BufferedReaderWrapper + ?Sized>,
    chunk_size_cdps: usize,
) -> Result<CdpChunk<T>, std::io::Error> {
    let mut cdp_chunk = CdpChunk::new();

    for _ in 0..chunk_size_cdps {
        let cdp_tuple = match file_scanner.load_cdp() {
            Ok(cdp) => cdp,
            Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
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

#[derive(Debug, Clone, PartialEq)]
/// Possible stats that can be sent to the StatsController.
pub enum InputStatType {
    /// Fatal error, stop processing.
    Fatal(String),
    /// The first trigger type observed is the type of run the data comes from
    ///
    /// Contains the raw value and the string description summarizing the trigger type
    RunTriggerType(u32),
    /// Record the data format detected.
    DataFormat(u8),
    /// Add a link to the list of links observed.
    LinksObserved(u8),
    /// Record the generic FEE ID
    FeeId(u16),
    /// Increment the total RDHs seen.
    RDHSeen,
    /// Increment the total RDHs filtered.
    RDHFiltered,
    /// Increment the total payload size.
    PayloadSize(u32),
    /// The first system ID observed is the basis for the rest of processing
    SystemId(u8),
}
