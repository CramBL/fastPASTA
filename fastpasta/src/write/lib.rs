//! Contains functionality for writing filtered data to disk or stdout.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use crossbeam_channel::Receiver;

use super::writer::BufferedWriter;
use super::writer::Writer;
use crate::config::inputoutput::InputOutputOpt;
use alice_daq_protocol_reader::prelude::CdpChunk;
use alice_daq_protocol_reader::prelude::RDH;

/// The size of the buffer used by the writer
///
/// This is the maximum amount of data that can be buffered before it is written to disk.
const BUFFER_SIZE: usize = 1024 * 1024; // 1MB buffer

/// Spawns a thread with the Writer running, and returns the thread handle.
pub fn spawn_writer<T: RDH + 'static>(
    config: &'static impl InputOutputOpt,
    stop_flag: Arc<AtomicBool>,
    data_channel: Receiver<CdpChunk<T>>,
) -> thread::JoinHandle<()> {
    let writer_thread = thread::Builder::new().name("Writer".to_string());
    writer_thread
        .spawn({
            let mut writer = BufferedWriter::<T>::new(config, BUFFER_SIZE);
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
