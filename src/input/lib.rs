use super::bufreader_wrapper::BufferedReaderWrapper;
use super::data_wrapper::CdpChunk;
use super::input_scanner::{InputScanner, ScanCDP};
use super::stdin_reader::StdInReaderSeeker;
use super::util::buf_reader_with_capacity;
use crate::util::config::Opt;
use crate::words;
use crate::words::lib::RDH;
use crossbeam_channel::Receiver;
use std::sync::atomic::{AtomicBool, Ordering};

#[inline]
pub fn init_reader(config: &Opt) -> Result<Box<dyn BufferedReaderWrapper>, std::io::Error> {
    if let Some(path) = config.file() {
        log::trace!("Reading from file: {:?}", &path);
        let f = std::fs::OpenOptions::new().read(true).open(path)?;
        Ok(Box::new(buf_reader_with_capacity(f, 1024 * 50)))
    } else {
        log::trace!("Reading from stdin");
        if atty::is(atty::Stream::Stdin) {
            log::error!("stdin not redirected!");
        }
        Ok(Box::new(StdInReaderSeeker {
            reader: std::io::stdin(),
        }))
    }
}

#[inline]
pub fn get_chunk<T: words::lib::RDH>(
    file_scanner: &mut InputScanner<impl BufferedReaderWrapper + ?Sized>,
    chunk_size_cdps: usize,
) -> Result<CdpChunk<T>, std::io::Error> {
    let mut cdp_chunk = CdpChunk::new();

    for _ in 0..chunk_size_cdps {
        let cdp_tuple = match file_scanner.load_cdp() {
            Ok(cdp) => cdp,
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

pub fn spawn_reader<T: RDH + 'static>(
    stop_flag: std::sync::Arc<AtomicBool>,
    input_scanner: InputScanner<impl BufferedReaderWrapper + ?Sized + std::marker::Send + 'static>,
) -> (std::thread::JoinHandle<()>, Receiver<CdpChunk<T>>) {
    let reader_thread = std::thread::Builder::new().name("Reader".to_string());
    let (send_channel, rcv_channel) = crossbeam_channel::bounded(crate::CHANNEL_CDP_CAPACITY);
    let thread_handle = reader_thread
        .spawn({
            move || {
                let mut input_scanner = input_scanner;

                // Automatically extracts link to filter if one is supplied
                loop {
                    if stop_flag.load(Ordering::SeqCst) {
                        log::trace!("Stopping reader thread");
                        break;
                    }
                    let cdps = match get_chunk::<T>(&mut input_scanner, 100) {
                        Ok(cdp) => cdp,
                        Err(e) => {
                            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                                break;
                            } else if e.kind() == std::io::ErrorKind::InvalidData {
                                log::trace!(
                                    "Input scanner returned invalid data, exiting reader thread"
                                );
                                break;
                            } else {
                                panic!("Unexpected Error reading CDP chunks: {e}");
                            }
                        }
                    };

                    // Send a chunk to the checker
                    if let Err(e) = send_channel.try_send(cdps) {
                        if e.is_full() {
                            log::trace!("Checker is too slow");
                            if send_channel.send(e.into_inner()).is_err()
                                && !stop_flag.load(Ordering::SeqCst)
                            {
                                log::trace!("Unexpected error while sending data to checker");
                                break;
                            }
                        } else if stop_flag.load(Ordering::SeqCst) {
                            log::trace!("Stopping reader thread");
                            break;
                        }
                    }
                }
            }
        })
        .expect("Failed to spawn reader thread");
    (thread_handle, rcv_channel)
}
