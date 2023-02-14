use std::{
    io::{Read, Write},
    vec,
};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use fastpasta::{
    buf_reader_with_capacity, data_words::rdh::RdhCRUv7, file_open_read_only, ByteSlice, GbtWord,
};
pub struct RelativeOffset(i64);
impl RelativeOffset {
    fn new(byte_offset: u64) -> Self {
        RelativeOffset(byte_offset as i64)
    }
}

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

#[inline]

fn bench_buffer_capacity(c: &mut Criterion) {
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
    group.finish();
}

#[inline]
fn parse_rdh_manual(rdh_cru_size_bytes: u64, filename: &str, iterations: usize) {
    let filepath = std::path::PathBuf::from(filename);
    let file = file_open_read_only(&filepath).unwrap();
    let mut buf_reader = std::io::BufReader::new(file);
    for _i in 1..iterations {
        let rdh_tmp = RdhCRUv7::load(&mut buf_reader).expect("Failed to load RdhCRUv7");
        let relative_offset =
            RelativeOffset::new((rdh_tmp.offset_new_packet as u64) - rdh_cru_size_bytes);
        buf_reader
            .seek_relative(relative_offset.0)
            .expect("Error seeking");
        if rdh_tmp.rdh0.header_id != 7 {
            println!("WRONG header ID: {}", rdh_tmp.rdh0.header_id);
        }
    }
}

fn bench_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("deserialization");
    const RDH_CRU_SIZE_BYTES: u64 = 64;
    let filename = "../fastpasta_test_files/data_ols_its-ul-v0.5_3.4GB";
    for i in [1000, 10000, 50000].iter() {
        group.bench_with_input(BenchmarkId::new("manual", i.to_string()), i, |b, i| {
            b.iter(|| {
                parse_rdh_manual(
                    black_box(RDH_CRU_SIZE_BYTES),
                    black_box(filename),
                    black_box(*i),
                )
            })
        });
    }
    group.finish();
}

#[inline]
fn write_rdh_manual(rdhs: Vec<RdhCRUv7>, filename: &str) {
    let filepath = std::path::PathBuf::from(filename);
    let file = std::fs::File::options()
        .write(true)
        .create(true)
        .open(&filepath)
        .unwrap();
    let mut buf_writer = std::io::BufWriter::new(file);
    rdhs.into_iter().for_each(|rdh| {
        buf_writer.write_all(rdh.to_byte_slice()).unwrap();
    });
}

fn bench_serialization_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization_write");
    let filename = "../fastpasta_test_files/data_ols_ul.raw";
    const RDH_CRU_SIZE_BYTES: u64 = 64;
    let filepath = std::path::PathBuf::from(filename);
    let file = file_open_read_only(&filepath).unwrap();
    let mut buf_reader = std::io::BufReader::new(file);
    let rdhs: Vec<RdhCRUv7> = (0..50000)
        .map(|_| {
            let rdh_tmp = RdhCRUv7::load(&mut buf_reader).expect("Failed to load RdhCRUv7");
            let relative_offset =
                RelativeOffset::new((rdh_tmp.offset_new_packet as u64) - RDH_CRU_SIZE_BYTES);
            buf_reader
                .seek_relative(relative_offset.0)
                .expect("Error seeking");
            rdh_tmp
        })
        .collect();
    let filename_manual = "manualrds.raw";

    group.bench_with_input(BenchmarkId::new("manual", ""), filename_manual, |b, f| {
        b.iter(|| write_rdh_manual(black_box(rdhs.clone()), black_box(f)))
    });
    // cleanup
    let filepath = std::path::PathBuf::from(filename_manual);
    std::fs::remove_file(&filepath).unwrap();

    group.finish();
}

// Sanity checking on RDHs
#[inline]
fn sanity_check_rdhs(rdh_cru_size_bytes: u64, filename: &str, iterations: usize) {
    let filepath = std::path::PathBuf::from(filename);
    let file = file_open_read_only(&filepath).unwrap();
    let mut buf_reader = std::io::BufReader::new(file);
    let rdh_validator = fastpasta::validators::rdh::RDH_CRU_V7_VALIDATOR;
    let mut rdhs = 0;

    loop {
        let tmp_rdh = match RdhCRUv7::load(&mut buf_reader) {
            Ok(rdh) => rdh,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                print!("EOF reached! ");
                break;
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        };
        let relative_offset =
            RelativeOffset::new((tmp_rdh.offset_new_packet as u64) - rdh_cru_size_bytes);
        buf_reader
            .seek_relative(relative_offset.0)
            .expect("Error seeking");
        match rdh_validator.sanity_check(&tmp_rdh) {
            Ok(_) => (),
            Err(e) => {
                println!("Error: {}", e);
            }
        }
        rdhs += 1;
        if rdhs == iterations {
            break;
        }
    }
}

fn bench_rdh_sanity_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("rdh_sanity_check");
    const RDH_CRU_SIZE_BYTES: u64 = 64;
    let filename = "../fastpasta_test_files/data_ols_its-ul-v0.5_3.4GB";
    for i in [1000, 10000, 50000, 100000, 1000000].iter() {
        group.bench_with_input(BenchmarkId::new("manual", i.to_string()), i, |b, i| {
            b.iter(|| {
                sanity_check_rdhs(
                    black_box(RDH_CRU_SIZE_BYTES),
                    black_box(filename),
                    black_box(*i),
                )
            })
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_buffer_capacity,
    bench_deserialization,
    bench_serialization_write,
    bench_rdh_sanity_check
);
criterion_main!(benches);
