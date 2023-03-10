#[inline(always)]
pub fn buf_reader_with_capacity<R: std::io::Read>(
    input: R,
    capacity: usize,
) -> std::io::BufReader<R> {
    std::io::BufReader::with_capacity(capacity, input)
}
