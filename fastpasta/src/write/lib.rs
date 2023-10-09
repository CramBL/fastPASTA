//! Contains functionality for writing filtered data to disk or stdout.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use alice_protocol_reader::cdp_wrapper::cdp_array::CdpArray;
use crossbeam_channel::Receiver;

use super::writer::BufferedWriter;
use super::writer::Writer;
use crate::config::inputoutput::InputOutputOpt;
use alice_protocol_reader::prelude::RDH;

/// The size of the buffer used by the writer
///
/// This is the maximum amount of data that can be buffered before it is written to disk.
const BUFFER_SIZE: usize = 1024 * 1024; // 1MB buffer

/// Spawns a thread with the Writer running, and returns the thread handle.
pub fn spawn_writer<T: RDH + 'static, const CAP: usize>(
    config: &'static impl InputOutputOpt,
    stop_flag: Arc<AtomicBool>,
    data_recv: Receiver<CdpArray<T, CAP>>,
) -> thread::JoinHandle<()> {
    let writer_thread = thread::Builder::new().name("Writer".to_string());
    writer_thread
        .spawn({
            let mut writer = BufferedWriter::<T>::new(config, BUFFER_SIZE);
            move || loop {
                // Receive batch from checker
                let cdps = match data_recv.recv() {
                    Ok(cdps) => cdps,
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
                writer.push_cdp_arr(cdps);
            }
        })
        .expect("Failed to spawn writer thread")
}
