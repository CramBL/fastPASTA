//! Writes data to file/stdout. Uses a buffer to reduce the amount of syscalls.
//!
//! Receives data incrementally and once a certain amount is reached, it will
//! write it out to file/stdout.
//! Implements drop to flush the remaining data to the file once processing is done.

use crate::config::inputoutput::InputOutputOpt;
use alice_protocol_reader::prelude::CdpChunk;
use alice_protocol_reader::prelude::RDH;

/// Trait for a writer that can write ALICE readout data to file/stdout.
pub trait Writer<T: RDH> {
    /// Write data to file/stdout
    fn write(&mut self, data: &[u8]) -> std::io::Result<()>;
    /// Push a vector of RDHs to the buffer
    fn push_rdhs(&mut self, rdhs: Vec<T>);
    /// Push a vector of payloads to the buffer
    fn push_payload(&mut self, payload: Vec<u8>);
    /// Push a CDP chunk to the buffer
    fn push_cdp_chunk(&mut self, cdp_chunk: CdpChunk<T>);
    /// Flush the buffer to file/stdout
    fn flush(&mut self) -> std::io::Result<()>;
}

/// A writer that uses a buffer to reduce the amount of syscalls.
pub struct BufferedWriter<T: RDH> {
    filtered_rdhs_buffer: Vec<T>,
    filtered_payload_buffers: Vec<Vec<u8>>, // 1 Linked list per payload
    buf_writer: Option<std::io::BufWriter<std::fs::File>>, // If no file is specified -> write to stdout
    max_buffer_size: usize,
}

impl<T: RDH> BufferedWriter<T> {
    /// Create a new BufferedWriter from a config and a max buffer size.
    pub fn new(config: &impl InputOutputOpt, max_buffer_size: usize) -> Self {
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
    fn push_payload(&mut self, payload: Vec<u8>) {
        if self.filtered_payload_buffers.len() + 1 >= self.max_buffer_size {
            self.flush().expect("Failed to flush buffer");
        }
        self.filtered_payload_buffers.push(payload);
    }

    #[inline]
    fn push_cdp_chunk(&mut self, cdp_chunk: CdpChunk<T>) {
        if (self.filtered_rdhs_buffer.len() + cdp_chunk.len() >= self.max_buffer_size)
            || (self.filtered_payload_buffers.len() + cdp_chunk.len() >= self.max_buffer_size)
        {
            self.flush().expect("Failed to flush buffer");
        }
        cdp_chunk.into_iter().for_each(|(rdh, payload, _)| {
            self.filtered_rdhs_buffer.push(rdh);
            self.filtered_payload_buffers.push(payload);
        });
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

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::config::check::CheckCommands;
    use crate::config::inputoutput::DataOutputMode;
    use crate::config::test_util::MockConfig;
    use crate::config::Cfg;
    use alice_protocol_reader::prelude::test_data::CORRECT_RDH_CRU_V7;
    use alice_protocol_reader::prelude::{RdhCru, V6, V7};
    use clap::Parser;
    use temp_dir::TempDir;

    use super::*;

    const INPUT_FILE_STR: &str = "tests/test-data/10_rdh.raw";
    const CONFIG_STR_NEEDS_OUTPUT: [&str; 4] =
        ["fastpasta", "tests/test-data/10_rdh.raw", "-f", "2"];

    fn build_test_config(output_path: &std::path::Path) -> MockConfig {
        let mut cfg = MockConfig::new();
        cfg.check = Some(CheckCommands::Sanity { system: None });
        cfg.output = Some(output_path.to_owned());
        cfg.output_mode = DataOutputMode::File(output_path.into());
        cfg.input_file = Some(std::path::PathBuf::from(INPUT_FILE_STR));
        cfg.filter_link = Some(2);
        cfg
    }

    #[test]
    fn test_buffered_writer() {
        let tmp_d = TempDir::new().unwrap();
        let test_file_path = tmp_d.child("test.raw");
        let cfg = build_test_config(&test_file_path);
        {
            let writer = BufferedWriter::<RdhCru<V6>>::new(&cfg, 10);

            assert!(writer.buf_writer.is_some());
        }
    }

    #[test]
    #[should_panic]
    // Should panic, Because when the writer is dropped, it flushes the buffer, which will panic because the number of RDHs and payloads are not equal
    // Empty payloads are counted.
    fn test_push_2_rdh_v7_buffer_is_2() {
        let tmp_d = TempDir::new().unwrap();
        let test_file_path = tmp_d.child("test.raw");
        let mut config_str = CONFIG_STR_NEEDS_OUTPUT.to_vec();
        config_str.push("-o");
        config_str.push(test_file_path.to_str().unwrap());

        println!("config_str: {:?}", config_str);
        let config: Cfg = <Cfg>::parse_from(config_str);
        let rdhs = vec![CORRECT_RDH_CRU_V7, CORRECT_RDH_CRU_V7];
        let length = rdhs.len();
        println!("length: {}", length);
        {
            let mut writer = BufferedWriter::<RdhCru<V7>>::new(&config, 10);
            writer.push_rdhs(rdhs);
            let buf_size = writer.filtered_rdhs_buffer.len();
            println!("buf_size: {}", buf_size);
            assert_eq!(buf_size, length);
        }
    }

    #[test]
    fn test_push_2_rdh_v7_and_empty_payloads_buffers_are_2() {
        let tmp_d = TempDir::new().unwrap();
        let test_file_path = tmp_d.child("test.raw");
        let mut config_str = CONFIG_STR_NEEDS_OUTPUT.to_vec();
        config_str.push("-o");
        config_str.push(test_file_path.to_str().unwrap());

        let config: Cfg = <Cfg>::parse_from(config_str);

        let mut cdp_chunk = CdpChunk::new();

        cdp_chunk.push(CORRECT_RDH_CRU_V7, vec![0; 10], 0);
        cdp_chunk.push(CORRECT_RDH_CRU_V7, vec![0; 10], 0x40);

        let length = cdp_chunk.len();
        {
            let mut writer = BufferedWriter::<RdhCru<V7>>::new(&config, 10);
            writer.push_cdp_chunk(cdp_chunk);
            let buf_size = writer.filtered_rdhs_buffer.len();
            assert_eq!(buf_size, length);
        }
    }
}
