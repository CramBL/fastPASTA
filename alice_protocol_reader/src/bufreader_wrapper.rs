//! Wrapper trait for [std::io::BufReader], requires that the reader implements [std::io::Read] and [std::io::Seek]
use std::fs::File;
use std::io;

/// Allows a stdin reader to be used in the same way as a file reader, by making it possible to seek (skip data)
/// Formally it it requires implementing [std::io::Seek] but practically only the seek_relative method is used
/// and as such all other methods can be left unimplemented (return not implemented error)
pub trait BufferedReaderWrapper: io::Read + io::Seek + Send {
    /// Seek relative to the current position
    fn seek_relative(&mut self, offset: i64) -> io::Result<()>;
}

impl BufferedReaderWrapper for io::BufReader<File> {
    #[inline(always)]
    fn seek_relative(&mut self, offset: i64) -> io::Result<()> {
        self.seek_relative(offset)
    }
}

impl<T> BufferedReaderWrapper for &mut T
where
    T: BufferedReaderWrapper + std::marker::Sync,
{
    fn seek_relative(&mut self, offset: i64) -> io::Result<()> {
        (*self).seek_relative(offset)
    }
}

impl<T> BufferedReaderWrapper for Box<T>
where
    T: BufferedReaderWrapper + std::marker::Sync,
{
    fn seek_relative(&mut self, offset: i64) -> io::Result<()> {
        (*self).seek_relative(offset)
    }
}
