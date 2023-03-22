/// Wrapper trait for std::io::BufReader, requires that the reader implements Read and seek_relative
///
/// Allows a stdin reader to be used in the same way as a file reader, by making it possible to seek (skip data)
pub trait BufferedReaderWrapper: std::io::Read + std::io::Seek + Send {
    fn seek_relative(&mut self, offset: i64) -> std::io::Result<()>;
}

impl BufferedReaderWrapper for std::io::BufReader<std::fs::File> {
    fn seek_relative(&mut self, offset: i64) -> std::io::Result<()> {
        self.seek_relative(offset)
    }
}
