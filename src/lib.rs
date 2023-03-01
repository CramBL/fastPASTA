use std::{fmt::Display, fs::File, io::Write, path::PathBuf};

use util::config::Opt;
use words::rdh::RdhCRUv7;
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

#[inline]
pub fn file_open_read_only(path: &PathBuf) -> std::io::Result<std::fs::File> {
    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(path)
        .expect("File not found");
    Ok(file)
}

/// Only use
#[inline]
pub fn file_open_append(path: &PathBuf) -> std::io::Result<std::fs::File> {
    let file = File::options().append(true).open(path)?;
    Ok(file)
}

#[inline(always)]
pub fn buf_reader_with_capacity<R: std::io::Read>(
    input: R,
    capacity: usize,
) -> std::io::BufReader<R> {
    std::io::BufReader::with_capacity(capacity, input)
}

pub fn setup_buffered_reading(config: &Opt) -> std::io::BufReader<std::fs::File> {
    const CAPACITY: usize = 1024 * 10; // 10 KB
    let file = file_open_read_only(config.file().as_ref().expect("No file path in config"))
        .expect("Failed to open file");
    buf_reader_with_capacity(file, CAPACITY)
}

pub struct FilterLink {
    link_to_filter: u8,
    output: Option<File>, // If no file is specified -> write to stdout
    pub max_buffer_size: usize,
    pub filtered_rdhs_buffer: Vec<RdhCRUv7>,
    pub filtered_payload_buffers: Vec<Vec<u8>>, // 1 Linked list per payload
    total_filtered: u64,
}
impl FilterLink {
    pub fn new(config: &Opt, max_buffer_size: usize) -> Self {
        let f = match config.output() {
            Some(path) => {
                let path: PathBuf = path.to_owned();
                // Likely better to use File::create_new() but it's not stable yet
                let mut _f = File::create(path.to_owned()).expect("Failed to create output file");
                let file = file_open_append(&path).expect("Failed to open output file");
                Some(file)
            }
            None => None,
        };

        FilterLink {
            link_to_filter: config.filter_link().expect("No link to filter specified"),
            output: f,
            filtered_rdhs_buffer: vec![],
            max_buffer_size,
            filtered_payload_buffers: Vec::with_capacity(1024), // 1 KB capacity to prevent frequent reallocations
            total_filtered: 0,
        }
    }
    pub fn filter_link<T: std::io::Read>(&mut self, buf_reader: &mut T, rdh: RdhCRUv7) -> bool {
        if rdh.link_id == self.link_to_filter {
            // Read the payload of the RDH
            self.read_payload(buf_reader, rdh.memory_size as usize)
                .expect("Failed to read from buffer");

            if self.filtered_rdhs_buffer.len() > self.max_buffer_size {
                self.flush();
            }
            self.filtered_rdhs_buffer.push(rdh);
            self.total_filtered += 1;
            true
        } else {
            false
        }
    }
    fn flush(&mut self) {
        if self.filtered_rdhs_buffer.len() > 0 {
            if self.filtered_rdhs_buffer.len() != self.filtered_payload_buffers.len() {
                panic!("Number of RDHs and payloads don't match!");
            }
            if self.output.is_some() {
                // Write RDHs and payloads to file by zip iterator (RDH, payload)
                self.filtered_rdhs_buffer
                    .iter()
                    .zip(self.filtered_payload_buffers.iter())
                    .for_each(|(rdh, payload)| {
                        self.output
                            .as_ref()
                            .unwrap()
                            .write_all(rdh.to_byte_slice())
                            .unwrap();
                        self.output.as_ref().unwrap().write_all(payload).unwrap();
                    });
            } else {
                // Write RDHs and payloads to stdout by zip iterator (RDH, payload)
                self.filtered_rdhs_buffer
                    .iter()
                    .zip(self.filtered_payload_buffers.iter())
                    .for_each(|(rdh, payload)| {
                        std::io::stdout().write_all(rdh.to_byte_slice()).unwrap();
                        std::io::stdout().write_all(payload).unwrap();
                    });
            }
            self.filtered_rdhs_buffer.clear();
            self.filtered_payload_buffers.clear();
        }
    }

    fn read_payload<T: std::io::Read>(
        &mut self,
        buf_reader: &mut T,
        payload_size: usize,
    ) -> Result<(), std::io::Error> {
        let payload_size = payload_size - 64; // RDH size in bytes
        let mut payload: Vec<u8> = vec![0; payload_size];
        buf_reader
            .read_exact(&mut payload)
            .expect("Failed to read payload");
        self.filtered_payload_buffers.push(payload);
        Ok(())
    }
    pub fn print_stats(&self) {
        log::info!("Total filtered RDHs: {}", self.total_filtered);
    }
}

impl Drop for FilterLink {
    fn drop(&mut self) {
        self.flush();
    }
}
