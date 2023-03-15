pub mod data_write;
pub mod input;
pub mod stats;
pub mod util;
pub mod validators;
pub mod words;

// Larger capacity means less overhead, but more memory usage
// Too small capacity will cause the producer thread to block
// Too large capacity will cause down stream consumers to block
pub const CHANNEL_CDP_CAPACITY: usize = 100;

// 1. Setup reading (file or stdin)
// 2. Do checks on read data
// 3. Write data out (file or stdout)
pub fn process<T: words::lib::RDH + 'static>(
    config: std::sync::Arc<util::config::Opt>,
    loader: input::input_scanner::InputScanner<
        impl input::bufreader_wrapper::BufferedReaderWrapper + ?Sized + std::marker::Send + 'static,
    >,
    send_stats_ch: std::sync::mpsc::Sender<stats::stats_controller::StatType>,
    thread_stopper: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> std::io::Result<()> {
    // 1. Read data from file
    let (reader_handle, reader_rcv_channel) =
        input::lib::spawn_reader(thread_stopper.clone(), loader);

    // 2. Do checks on a received chunk of data
    let (validator_handle, checker_rcv_channel) = validators::lib::spawn_validator::<T>(
        config.clone(),
        thread_stopper.clone(),
        send_stats_ch,
        reader_rcv_channel,
    );

    // 3. Write data out
    let writer_handle: Option<std::thread::JoinHandle<()>> = match config.output_mode() {
        util::config::DataOutputMode::None => None,
        _ => Some(data_write::lib::spawn_writer(
            config.clone(),
            thread_stopper,
            checker_rcv_channel.expect("Checker receiver channel not initialized"),
        )),
    };

    reader_handle.join().expect("Error joining reader thread");
    if let Err(e) = validator_handle.join() {
        log::error!("Validator thread terminated early: {:#?}\n", e);
    }
    if let Some(writer) = writer_handle {
        writer.join().expect("Could not join writer thread");
    }
    Ok(())
}
