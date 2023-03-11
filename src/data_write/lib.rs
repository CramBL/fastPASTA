use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use crossbeam_channel::Receiver;

use super::writer::BufferedWriter;
use super::writer::Writer;
use crate::input::data_wrapper::CdpChunk;
use crate::util::config::Opt;
use crate::words::rdh::RDH;

const BUFFER_SIZE: usize = 1024 * 1024; // 1MB buffer

pub fn spawn_writer<T: RDH + 'static>(
    config: Arc<Opt>,
    stop_flag: Arc<AtomicBool>,
    data_channel: Receiver<CdpChunk<T>>,
) -> thread::JoinHandle<()> {
    let writer_thread = thread::Builder::new().name("Writer".to_string());
    writer_thread
        .spawn({
            let mut writer = BufferedWriter::<T>::new(&config, BUFFER_SIZE);
            move || loop {
                // Receive chunk from checker
                let cdps = match data_channel.recv() {
                    Ok(cdp) => cdp,
                    Err(e) => {
                        debug_assert_eq!(e, crossbeam_channel::RecvError);
                        break;
                    }
                };
                if stop_flag.load(Ordering::SeqCst) {
                    log::trace!("Stopping writer thread");
                    break;
                }
                // Push data onto the writer's buffer, which will flush it when the buffer is full or when the writer is dropped
                writer.push_cdp_chunk(cdps);
            }
        })
        .expect("Failed to spawn writer thread")
}
