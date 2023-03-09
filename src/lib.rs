use input::{bufreader_wrapper::BufferedReaderWrapper, input_scanner::InputScanner};
use std::{fmt::Display, sync::atomic::AtomicBool};
use util::{config::Opt, stats_controller::Stats};

use crate::input::stdin_reader::StdInReaderSeeker;
pub mod input;
mod stats;
pub mod util;
pub mod validators;
pub mod words;

/// This is the trait that all GBT words must implement
/// It is used to:
/// * pretty printing to stdout
/// * deserialize the GBT words from the binary file
pub trait GbtWord: std::fmt::Debug + PartialEq + Sized + Display {
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized;
}

/// This trait is used to convert a struct to a byte slice
/// All structs that are used to represent a full GBT word (not sub RDH words) must implement this trait
pub trait ByteSlice {
    fn to_byte_slice(&self) -> &[u8];
}

/// # Safety
/// This function can only be used to serialize a struct if it has the #[repr(packed)] attribute
/// If there's any padding on T, it is UNITIALIZED MEMORY and therefor UNDEFINED BEHAVIOR!
#[inline]
pub unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    // Create read-only reference to T as a byte slice, safe as long as no padding bytes are read
    ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
}

pub fn init_stats_controller(
    config: &Opt,
) -> (
    std::thread::JoinHandle<()>,
    std::sync::mpsc::Sender<util::stats_controller::StatType>,
    std::sync::Arc<AtomicBool>,
) {
    let (send_stats_channel, recv_stats_channel): (
        std::sync::mpsc::Sender<util::stats_controller::StatType>,
        std::sync::mpsc::Receiver<util::stats_controller::StatType>,
    ) = std::sync::mpsc::channel();
    let thread_stop_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mut stats = Stats::new(config, recv_stats_channel, thread_stop_flag.clone());
    let stats_thread = std::thread::Builder::new()
        .name("stats_thread".to_string())
        .spawn(move || {
            stats.run();
        })
        .expect("Failed to spawn stats thread");
    (stats_thread, send_stats_channel, thread_stop_flag)
}

pub fn init_reader(config: &Opt) -> Box<dyn BufferedReaderWrapper> {
    match config.file() {
        Some(path) => {
            log::trace!("Reading from file: {:?}", &path);
            let f = std::fs::OpenOptions::new()
                .read(true)
                .open(path)
                .expect("File not found");
            Box::new(buf_reader_with_capacity(f, 1024 * 50))
        }
        None => {
            log::trace!("Reading from stdin");
            if atty::is(atty::Stream::Stdin) {
                log::trace!("stdin not redirected!");
            }
            Box::new(StdInReaderSeeker {
                reader: std::io::stdin(),
            })
        }
    }
}

#[inline(always)]
pub fn buf_reader_with_capacity<R: std::io::Read>(
    input: R,
    capacity: usize,
) -> std::io::BufReader<R> {
    std::io::BufReader::with_capacity(capacity, input)
}

pub fn get_chunk<T: words::rdh::RDH>(
    file_scanner: &mut InputScanner<impl BufferedReaderWrapper + ?Sized>,
    chunk_size_cdps: usize,
) -> Result<(Vec<T>, Vec<Vec<u8>>), std::io::Error> {
    let mut rdhs: Vec<T> = vec![];
    let mut payloads: Vec<Vec<u8>> = vec![];

    use input::input_scanner::ScanCDP;

    for _ in 0..chunk_size_cdps {
        let (rdh, payload) = match file_scanner.load_cdp() {
            Ok(cdp) => cdp,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                log::info!("EOF reached! ");
                break;
            }
            Err(e) => return Err(e),
        };
        rdhs.push(rdh);
        payloads.push(payload);
    }

    if rdhs.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "No CDPs found",
        ));
    }

    Ok((rdhs, payloads))
}
