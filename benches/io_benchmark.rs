use std::io::Read;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use fastpasta::{buf_reader_with_capacity, file_open_read_only};

// As of 12.02.2023 the default is 8 KB, and may change in the future
#[inline]
fn buffered_read_default_capacity() {
    let path = std::path::PathBuf::from("../fastpasta_test_files/data_ols_ul.raw");
    let file = file_open_read_only(&path).unwrap();
    let mut buf_reader = std::io::BufReader::new(file);

    let mut ten_mb_vec = vec![0; 1024 * 1024 * 10];
    Read::read_exact(&mut buf_reader, &mut ten_mb_vec).unwrap();
}

#[inline]
fn buffered_read_custom_capacity(n: usize) {
    let path = std::path::PathBuf::from("../fastpasta_test_files/data_ols_ul.raw");
    let file = file_open_read_only(&path).unwrap();
    let mut buf_reader = buf_reader_with_capacity(file, n);
    let mut ten_mb_vec = vec![0; 1024 * 1024 * 10];
    Read::read_exact(&mut buf_reader, &mut ten_mb_vec).unwrap();
    buf_reader.read_to_end(&mut ten_mb_vec).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffered_read");
    const ONE_KB: usize = 1024;
    const TEN_KB: usize = 1024 * 10;
    const FIFTY_KB: usize = 1024 * 50;
    const HUNDRED_KB: usize = 1024 * 100;
    const ONE_MB: usize = 1024 * 1024;
    const TEN_MB: usize = 1024 * 1024 * 10;
    const FIFTY_MB: usize = 1024 * 1024 * 50;
    for i in [
        ONE_KB, TEN_KB, FIFTY_KB, HUNDRED_KB, ONE_MB, TEN_MB, FIFTY_MB,
    ]
    .iter()
    {
        group.bench_with_input(BenchmarkId::new("buffered_read_", i), i, |b, i| {
            b.iter(|| buffered_read_custom_capacity(*i))
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
