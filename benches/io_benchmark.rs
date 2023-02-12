use std::io::Read;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use fastpasta::{buf_reader_with_capacity, file_open_read_only};

// bybass lib arg parser with e.g.: cargo bench --bench io_benchmark -- --measurement-time 15
#[inline]
fn buffered_read_custom_capacity(n: usize) {
    let path = std::path::PathBuf::from("../fastpasta_test_files/data_ols_ul.raw");
    let file = file_open_read_only(&path).unwrap();
    let mut buf_reader = buf_reader_with_capacity(file, n);
    let mut ten_mb_vec = vec![0; 1024 * 1024 * 10];
    Read::read_exact(&mut buf_reader, &mut ten_mb_vec).unwrap();
    for _i in 0..14 {
        ten_mb_vec.clear();
        Read::read_exact(&mut buf_reader, &mut ten_mb_vec).unwrap();
    }
    buf_reader.read_to_end(&mut ten_mb_vec).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffered_read");
    static KB: usize = 1024;
    static MB: usize = 1024 * KB;
    static DEFAULT_8KB: usize = 8 * KB; // As of 12.02.2023 the default is 8 KB, and may change in the future

    for i in [
        1 * KB,
        DEFAULT_8KB,
        10 * KB,
        50 * KB,
        100 * KB,
        1 * MB,
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
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
