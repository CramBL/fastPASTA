use super::bufreader_wrapper::BufferedReaderWrapper;
use std::io::{self, Read, SeekFrom};

pub struct StdInReaderSeeker {
    pub reader: io::Stdin,
}

impl BufferedReaderWrapper for StdInReaderSeeker {
    fn seek_relative(&mut self, offset: i64) -> io::Result<()> {
        // Seeking is not supported in stdin, so we have to read the bytes and discard them
        let mut buf = vec![0; offset as usize];
        let _ = std::io::stdin().lock().read_exact(&mut buf)?;
        Ok(())
    }
}

impl io::Read for StdInReaderSeeker {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.lock().read(buf)
    }
}
impl io::Seek for StdInReaderSeeker {
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
