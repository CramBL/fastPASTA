//! Wrapper for a reader, implements [BufferedReaderWrapper].
//!
//! The wrapper can wrap both [BufReader](std::io::BufReader) and [StdInReaderSeeker].
//! Needed because [Stdin](std::io::Stdin) does not implement seek_relative, and this serves as a convenient way to skip unwanted data.
//! seek_relative is used to skip over unwanted bytes in the input stream, such as links unwanted by the user
use super::bufreader_wrapper::BufferedReaderWrapper;
use std::io::{self, Read, SeekFrom};

/// Wrapper for a reader where input data can be read from, implements [BufferedReaderWrapper].
#[derive(Debug)]
pub struct StdInReaderSeeker<R> {
    /// Generic reader that is wrapped
    pub reader: R,
}

/// Specialization for [std::io::Stdin]
impl BufferedReaderWrapper for StdInReaderSeeker<std::io::Stdin> {
    fn seek_relative(&mut self, offset: i64) -> io::Result<()> {
        // Seeking is not supported in stdin, so we have to read the bytes and discard them
        let mut buf = vec![0; offset as usize];
        match std::io::stdin().lock().read_exact(&mut buf) {
            Ok(_) => Ok(()),
            // If we're seeking the offset amount and reached an unexpected EOF then it's possible that the offset retrieved from the RDH is wrong
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                // Seeking past EOF is InvalidInput in this case
                Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Failed to read and discard a payload from stdin of size {offset} (according to previously loaded RDH): {e}"),
            ))
            }
            Err(e) => Err(e),
        }
    }
}

impl io::Read for StdInReaderSeeker<std::io::Stdin> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.lock().read(buf)
    }
}
impl io::Seek for StdInReaderSeeker<std::io::Stdin> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match pos {
            SeekFrom::Start(_) => Err(io::Error::new(
                io::ErrorKind::Other,
                "Cannot seek from start in stdin",
            )),
            SeekFrom::Current(_) => Err(io::Error::new(
                io::ErrorKind::Other,
                "Cannot seek from current in stdin, use seek_relative instead",
            )),
            SeekFrom::End(_) => Err(io::Error::new(
                io::ErrorKind::Other,
                "Cannot seek from end in stdin",
            )),
        }
    }
}
