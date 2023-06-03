use std::{
    io::{Read, Write},
    vec,
};

use criterion::{black_box, BenchmarkId, Criterion};
use fastpasta::words::lib::{ByteSlice, SerdeRdh, RDH_CRU};
use fastpasta::{words::rdh_cru::RdhCRU, words::rdh_cru::V7};
pub struct RelativeOffset(i64);
impl RelativeOffset {
    fn new(byte_offset: u64) -> Self {
        RelativeOffset(byte_offset as i64)
    }
}

#[inline]
pub fn buf_reader_with_capacity<R: std::io::Read>(
    input: R,
    capacity: usize,
) -> std::io::BufReader<R> {
    std::io::BufReader::with_capacity(capacity, input)
}

// bybass lib arg parser with e.g.: cargo bench --bench io_benchmark -- --measurement-time 15
#[inline]
fn buffered_read_custom_capacity(n: usize) {
    let path = std::path::PathBuf::from("../fastpasta_test_files/data_ols_ul.raw");
    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(&path)
        .expect("File not found");
    let mut buf_reader = buf_reader_with_capacity(file, n);
    let mut ten_mb_vec = vec![0; 1024 * 1024 * 10];
    Read::read_exact(&mut buf_reader, &mut ten_mb_vec).unwrap();
    for _i in 0..14 {
        ten_mb_vec.clear();
        Read::read_exact(&mut buf_reader, &mut ten_mb_vec).unwrap();
    }
    buf_reader.read_to_end(&mut ten_mb_vec).unwrap();
}

#[inline]

pub fn bench_buffer_capacity(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffered_read");
    static KB: usize = 1024;
    static MB: usize = 1024 * KB;
    static DEFAULT_8KB: usize = 8 * KB; // As of 12.02.2023 the default is 8 KB, and may change in the future

    for i in [
        KB,
        DEFAULT_8KB,
        10 * KB,
        50 * KB,
        100 * KB,
        MB,
        10 * MB,
        50 * MB,
    ]
    .iter()
    {
        group.bench_with_input(
            BenchmarkId::new("with_capacity", (i / KB).to_string() + "_KB"),
            i,
            |b, i| b.iter(|| buffered_read_custom_capacity(black_box(*i))),
        );
    }
    group.finish();
}
