#![forbid(unused_extern_crates)]
#![deny(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]
#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused_results)]
#![warn(unused_import_braces)]
#![warn(variant_size_differences)]
#![warn(
    clippy::option_filter_map,
    clippy::manual_filter_map,
    clippy::if_not_else,
    clippy::nonminimal_bool
)]
// Performance lints
#![warn(
    clippy::needless_pass_by_value,
    clippy::unnecessary_wraps,
    clippy::mutex_integer,
    clippy::mem_forget,
    clippy::maybe_infinite_iter
)]

//! This module contains mainly the [InputScanner] that reads the input data, and the [CdpArray] data structure that wraps the data read from the input.
//! Additionally it contains a helper function [spawn_reader] that spawns a thread that reads input and sents it to a channel that is returned from the function.
//!
//! The [InputScanner] is a generic type that can be instantiated with any type that implements the [BufferedReaderWrapper] trait.
//! This trait is implemented for the [StdInReaderSeeker] and the [BufReader](io::BufReader) types.
//! Allowing the [InputScanner] to read from both stdin and files, in a convenient and efficient way.
//!
//! The [CdpArray] is a wrapper for the data read from the input, it contains the data and the memory address of the first byte of the data.

//! # Example
//! First add the `alice_protocol_reader` crate to your project
//! ```shell
//! $ cargo add alice_protocol_reader
//! ```
//! Then use the convenience `init_reader()`-function to add the appropriate reader (stdin or file) at runtime. Instantiate the `InputScanner` with the reader and start reading ALICE data.
//! ```text
//! use alice_protocol_reader::input_scanner::InputScanner;
//! use alice_protocol_reader::init_reader;
//! use alice_protocol_reader::rdh::RdhCru;
//!
//! let reader = init_reader(&Some(test_file_path)).unwrap();
//!
//! let mut input_scanner = InputScanner::minimal(reader);
//!
//! let rdh = input_scanner.load_rdh_cru::<RdhCru<u8>>().unwrap();
//!
//! println!("{rdh:?}");
//! ```
//! Example output
//!
//! ```text
//! RdhCru
//!         Rdh0 { header_id: 7, header_size: 64, fee_id: 20522, priority_bit: 0, system_id: 32, reserved0: 0 }
//!         offset_new_packet: 5088
//!         memory_size: 5088
//!         link_id: 0
//!         packet_counter: 0
//!         cruid_dw: 24
//!         Rdh1 { bc_reserved0: 0, orbit: 192796021 }
//!         dataformat_reserved0: 2
//!         Rdh2 { trigger_type: 27139, pages_counter: 0, stop_bit: 0, reserved0: 0 }
//!         reserved1: 0
//!         Rdh3 { detector_field: 0, par_bit: 0, reserved0: 0 }
//!         reserved2: 0
//! ```
//!
//! ## Customize InputScanner behaviour with a config
//!
//! Implement the `FilterOpt` on your own config struct and pass it to the `InputScanner` to customize its behaviour
//!
//! ```text
//! use alice_protocol_reader::filter::FilterOpt;
//!
//! struct MyCfg;
//!
//! impl FilterOpt for MyCfg {
//!     fn skip_payload(&self) -> bool {
//!         // Implement your config rules for determining if you're skipping the payload (only reading `RDH`s)
//!     }
//!
//!     fn filter_link(&self) -> Option<u8> {
//!         // Implement your config rules for setting a link to filter by
//!     }
//!
//!     fn filter_fee(&self) -> Option<u16> {
//!         // Implement your config rules for setting a FEE ID to filter by
//!     }
//!
//!     fn filter_its_stave(&self) -> Option<u16> {
//!         // Implement your config rules for setting an ITS Stave to filter by
//!     }
//! }
//!
//! use alice_protocol_reader::input_scanner::InputScanner;
//! use alice_protocol_reader::init_reader;
//! use alice_protocol_reader::rdh::RdhCru;
//! pub fn main() {}
//!     let reader = init_reader(&Some(test_file_path)).unwrap();
//!
//!     let mut input_scanner = input_scanner::InputScanner::new(&MyCfg, reader, None); // None: Option<flume::Sender<InputStatType>>
//!
//!     let rdh = input_scanner.load_cdp::<RdhCru<u8>>();
//! }
//! ```

pub mod bufreader_wrapper;
pub mod cdp_wrapper;
pub mod config;
pub mod input_scanner;
pub mod mem_pos_tracker;
pub mod prelude;
pub mod rdh;
pub mod scan_cdp;
pub mod stats;
pub mod stdin_reader;

use cdp_wrapper::cdp_array::CdpArray;
use crossbeam_channel::Receiver;
use prelude::{BufferedReaderWrapper, CdpVec, InputScanner, ScanCDP, RDH};
use std::sync::Arc;
use std::thread::{Builder, JoinHandle};
use std::{fs, io};
use std::{
    io::IsTerminal,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};
use stdin_reader::StdInReaderSeeker;

/// Depth of the FIFO where the CDP batches are inserted as they are read
const CHANNEL_CDP_BATCH_CAPACITY: usize = 100;
const READER_BUFFER_SIZE: usize = 1024 * 50; // 50KB

/// Initializes the reader based on the input mode (file or stdin) and returns it
///
/// The input mode is determined by the presence of the input file path in the config
#[inline]
pub fn init_reader(input_file: Option<&Path>) -> Result<Box<dyn BufferedReaderWrapper>, io::Error> {
    if let Some(path) = input_file {
        let f = fs::OpenOptions::new().read(true).open(path)?;
        Ok(Box::new(io::BufReader::with_capacity(
            READER_BUFFER_SIZE,
            f,
        )))
    } else if !io::stdin().is_terminal() {
        Ok(Box::new(StdInReaderSeeker {
            reader: io::stdin(),
        }))
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "stdin not redirected!",
        ))
    }
}

/// Spawns a reader thread that reads CDPs from the input and sends them to a producer channel
///
/// Returns the thread handle and the receiver channel
///
/// If you want more control of the batch size at runtime, use [spawn_vec_reader] instead.
#[inline]
pub fn spawn_reader<T: RDH + 'static, const CAP: usize>(
    stop_flag: Arc<AtomicBool>,
    input_scanner: InputScanner<impl BufferedReaderWrapper + ?Sized + 'static>,
) -> (JoinHandle<()>, Receiver<CdpArray<T, CAP>>) {
    let reader_thread = Builder::new().name("Reader".to_string());
    let (send_chan, recv_chan) = crossbeam_channel::bounded(CHANNEL_CDP_BATCH_CAPACITY);
    let mut local_stop_on_non_full_batch = false;

    let thread_handle = reader_thread
        .spawn({
            move || {
                let mut input_scanner = input_scanner;

                // Automatically extracts link to filter if one is supplied
                while !stop_flag.load(Ordering::SeqCst) && !local_stop_on_non_full_batch {
                    let cdps = match get_array_batch::<T, CAP>(&mut input_scanner) {
                        Ok(cdp) => {
                            if cdp.len() < CAP {
                                local_stop_on_non_full_batch = true; // Stop on non-full batch, could be InvalidData
                            }
                            cdp
                        }
                        Err(_) => {
                            break;
                        }
                    };

                    // Send a batch to the checker
                    if send_chan.send(cdps).is_err() {
                        break;
                    }
                }
            }
        })
        .expect("Failed to spawn reader thread");
    (thread_handle, recv_chan)
}

/// Attempts to fill a CDP batch with as many CDPs as possible (up to the batch capacity) and returns it
///
/// If an error occurs after one or more CDPs have been read, the CDP batch is returned with the CDPs read so far
/// If the error occurs before any CDPs have been read, the error is returned
#[inline]
fn get_array_batch<T: RDH, const CAP: usize>(
    file_scanner: &mut InputScanner<impl BufferedReaderWrapper + ?Sized>,
) -> Result<CdpArray<T, CAP>, io::Error> {
    let mut cdp_arr = CdpArray::<T, CAP>::new_const();

    for _ in 0..CAP {
        let (rdh, payload, mem_pos) = match file_scanner.load_cdp() {
            Ok(cdp) => cdp,
            Err(e) if e.kind() == io::ErrorKind::InvalidData => {
                break;
            }
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(e) => return Err(e),
        };
        cdp_arr.push(rdh, payload, mem_pos);
    }

    if cdp_arr.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "No CDPs found",
        ));
    }

    Ok(cdp_arr)
}

/// Spawns a reader thread that reads CDPs from the input and sends them to a producer channel
///
/// Returns the thread handle and the receiver channel
#[inline]
pub fn spawn_vec_reader<T: RDH + 'static>(
    stop_flag: Arc<AtomicBool>,
    input_scanner: InputScanner<impl BufferedReaderWrapper + ?Sized + 'static>,
) -> (JoinHandle<()>, Receiver<CdpVec<T>>) {
    let reader_thread = Builder::new().name("Reader".to_string());
    let (send_chan, recv_chan) = crossbeam_channel::bounded(CHANNEL_CDP_BATCH_CAPACITY);
    let mut local_stop_on_non_full_batch = false;
    const CDP_BATCH_SIZE: usize = 100;
    let thread_handle = reader_thread
        .spawn({
            move || {
                let mut input_scanner = input_scanner;

                // Automatically extracts link to filter if one is supplied
                while !stop_flag.load(Ordering::SeqCst) && !local_stop_on_non_full_batch {
                    let cdps = match get_vec_batch::<T>(&mut input_scanner, CDP_BATCH_SIZE) {
                        Ok(cdp) => {
                            if cdp.len() < CDP_BATCH_SIZE {
                                local_stop_on_non_full_batch = true; // Stop on non-full batch, could be InvalidData
                            }
                            cdp
                        }
                        Err(_) => {
                            break;
                        }
                    };

                    // Send a batch to the checker
                    if send_chan.send(cdps).is_err() {
                        break;
                    }
                }
            }
        })
        .expect("Failed to spawn reader thread");
    (thread_handle, recv_chan)
}

/// Attempts to fill a CDP batch with as many CDPs as possible (up to the batch size) and returns it.
///
/// If an error occurs after one or more CDPs have been read, the CDP batch is returned with the CDPs read so far
/// If the error occurs before any CDPs have been read, the error is returned
#[inline]
fn get_vec_batch<T: RDH>(
    file_scanner: &mut InputScanner<impl BufferedReaderWrapper + ?Sized>,
    batch_size_cdps: usize,
) -> Result<CdpVec<T>, io::Error> {
    let mut cdp_batch = CdpVec::with_capacity(batch_size_cdps);

    for _ in 0..batch_size_cdps {
        let cdp_tuple = match file_scanner.load_cdp() {
            Ok(cdp) => cdp,
            Err(e) if e.kind() == io::ErrorKind::InvalidData => {
                break;
            }
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                // This error will always be returned when the input is exhausted
                //  as we try to read a CDP from the input without knowing if there is one
                break;
            }
            Err(e) => return Err(e),
        };
        cdp_batch.push(cdp_tuple.0, cdp_tuple.1, cdp_tuple.2);
    }

    if cdp_batch.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "No CDPs found",
        ));
    }

    Ok(cdp_batch)
}

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    use super::*;
    use rdh::test_data::CORRECT_RDH_CRU_V7;
    use rdh::test_data::CORRECT_RDH_CRU_V7_NEXT;
    use rdh::test_data::CORRECT_RDH_CRU_V7_NEXT_NEXT_STOP;
    use rdh::ByteSlice;
    use temp_dir::TempDir;

    #[test]
    fn test_minimal() {
        let tmp_d = TempDir::new().unwrap();
        let test_file_path = tmp_d.child("test.raw");
        let test_data = CORRECT_RDH_CRU_V7;
        println!("Test data: \n{test_data:?}");
        // Write to file for testing
        std::fs::write(&test_file_path, CORRECT_RDH_CRU_V7.to_byte_slice()).unwrap();

        use crate::input_scanner::InputScanner;
        use rdh::RdhCru;
        let reader = init_reader(Some(&test_file_path)).unwrap();

        let mut input_scanner = InputScanner::minimal(reader);

        let rdh = input_scanner.load_rdh_cru::<RdhCru>().unwrap();

        println!("{rdh:?}");
    }

    use config::filter::FilterOpt;

    struct MyCfg;

    impl FilterOpt for MyCfg {
        fn skip_payload(&self) -> bool {
            false
        }

        fn filter_link(&self) -> Option<u8> {
            None
        }

        fn filter_fee(&self) -> Option<u16> {
            None
        }

        fn filter_its_stave(&self) -> Option<u16> {
            None
        }
    }

    #[test]
    fn test_with_custom_config() {
        let tmp_d = TempDir::new().unwrap();
        let test_file_path = tmp_d.child("test.raw");
        let test_data = CORRECT_RDH_CRU_V7;
        println!("Test data: \n{test_data:?}");
        // Write to file for testing
        std::fs::write(&test_file_path, CORRECT_RDH_CRU_V7.to_byte_slice()).unwrap();

        use rdh::RdhCru;

        let reader = init_reader(Some(&test_file_path)).unwrap();

        let mut input_scanner = input_scanner::InputScanner::new(&MyCfg, reader, None);

        let rdh = input_scanner.load_rdh_cru::<RdhCru>();

        match rdh {
            Ok(rdh) => println!("{rdh:?}"),
            Err(e) => eprintln!("{e}"),
        }
    }
}
