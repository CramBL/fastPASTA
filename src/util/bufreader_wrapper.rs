pub trait BufferedReaderWrapper: std::io::Read + std::io::Seek + Send {
    fn seek_relative(&mut self, offset: i64) -> std::io::Result<()>;
}

impl BufferedReaderWrapper for std::io::BufReader<std::fs::File> {
    fn seek_relative(&mut self, offset: i64) -> std::io::Result<()> {
        self.seek_relative(offset)
    }
}
