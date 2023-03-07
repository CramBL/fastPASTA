use super::config::Opt;
use crate::words::rdh::RDH;
/// Writes data to file/stdout
/// Uses a buffer to minimize syscalls.
/// Receives data incrementally and once a certain amount is reached, it will
/// write it out to file/stdout.
/// Implements drop to flush the remaining data to the file once processing is done.
pub trait Writer<T> {
    fn write(&mut self, data: &[u8]) -> std::io::Result<()>;
    fn push_rdhs(&mut self, rdhs: Vec<T>);
    fn push_cdps_raw(&mut self, cdps: (Vec<T>, Vec<Vec<u8>>));
    fn flush(&mut self) -> std::io::Result<()>;
}

pub struct BufferedWriter<T: RDH> {
    pub filtered_rdhs_buffer: Vec<T>,
    pub filtered_payload_buffers: Vec<Vec<u8>>, // 1 Linked list per payload
    buf_writer: Option<std::io::BufWriter<std::fs::File>>, // If no file is specified -> write to stdout
    max_buffer_size: usize,
}

impl<T: RDH> BufferedWriter<T> {
    pub fn new(config: &Opt, max_buffer_size: usize) -> Self {
        // Create output file, and buf writer if specified
        let buf_writer = match config.output() {
            Some(path) if "stdout".eq(path.to_str().unwrap()) => None,
            Some(path) => {
                let path: std::path::PathBuf = path.to_owned();
                // Likely better to use File::create_new() but it's not stable yet
                let mut _f = std::fs::File::create(&path).expect("Failed to create output file");
                let file = std::fs::File::options()
                    .append(true)
                    .open(path)
                    .expect("Failed to open/create output file");
                let buf_writer = std::io::BufWriter::new(file);
                Some(buf_writer)
            }
            None => None,
        };
        BufferedWriter {
            filtered_rdhs_buffer: Vec::with_capacity(max_buffer_size), // Will most likely not be filled as payloads are usually larger, but hard to say
            filtered_payload_buffers: Vec::with_capacity(max_buffer_size),
            buf_writer,
            max_buffer_size,
        }
    }
}

impl<T: RDH> Writer<T> for BufferedWriter<T> {
    #[inline]
    fn write(&mut self, data: &[u8]) -> std::io::Result<()> {
        match &mut self.buf_writer {
            Some(buf_writer) => std::io::Write::write_all(buf_writer, data),
            None => std::io::Write::write_all(&mut std::io::stdout(), data),
        }
    }

    #[inline]
    fn push_rdhs(&mut self, rdhs: Vec<T>) {
        if self.filtered_rdhs_buffer.len() + rdhs.len() >= self.max_buffer_size {
            self.flush().expect("Failed to flush buffer");
        }
        self.filtered_rdhs_buffer.extend(rdhs);
    }

    #[inline]
    fn push_cdps_raw(&mut self, cdps: (Vec<T>, Vec<Vec<u8>>)) {
        self.filtered_rdhs_buffer.extend(cdps.0);
        self.filtered_payload_buffers.extend(cdps.1);
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        debug_assert_eq!(
            self.filtered_rdhs_buffer.len(),
            self.filtered_payload_buffers.len()
        );

        let mut data = vec![];
        for (rdh, payload) in self
            .filtered_rdhs_buffer
            .iter()
            .zip(self.filtered_payload_buffers.iter())
        {
            data.extend(rdh.to_byte_slice());
            data.extend(payload);
        }
        self.write(&data)?;
        self.filtered_rdhs_buffer.clear();
        self.filtered_payload_buffers.clear();
        Ok(())
    }
}

impl<T: RDH> Drop for BufferedWriter<T> {
    fn drop(&mut self) {
        if std::mem::needs_drop::<Self>() {
            self.flush().expect("Failed to flush buffer");
        }
    }
}
