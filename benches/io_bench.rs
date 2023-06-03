use std::{
    io::{Read, Write},
    vec,
};

use criterion::{black_box, BenchmarkId, Criterion};
use fastpasta::words::{
    lib::{ByteSlice, RdhSubWord, SerdeRdh, RDH_CRU},
    rdh::Rdh0,
};
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

#[inline]
fn parse_rdh_manual(rdh_cru_size_bytes: u64, filename: &str, iterations: usize) {
    let filepath = std::path::PathBuf::from(filename);
    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(&filepath)
        .expect("File not found");
    let mut buf_reader = std::io::BufReader::new(file);
    for _i in 1..iterations {
        let rdh_tmp: RdhCRU<V7> = SerdeRdh::load(&mut buf_reader).expect("Failed to load RdhCRUv7");
        let relative_offset =
            RelativeOffset::new((rdh_tmp.offset_to_next() as u64) - rdh_cru_size_bytes);
        buf_reader
            .seek_relative(relative_offset.0)
            .expect("Error seeking");
        if rdh_tmp.rdh0().header_id != 7 {
            println!("WRONG header ID: {}", rdh_tmp.rdh0().header_id);
        }
    }
}

pub fn bench_deserialization(c: &mut Criterion) {
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
fn write_rdh_manual(fileout: &str) {
    let filename = "../fastpasta_test_files/data_ols_ul.raw";
    const RDH_CRU_SIZE_BYTES: u64 = 64;
    let filepath = std::path::PathBuf::from(filename);
    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(filepath)
        .expect("File not found");
    let mut buf_reader = std::io::BufReader::new(file);
    let rdhs: Vec<RdhCRU<V7>> = (0..50000)
        .map(|_| {
            let rdh_tmp = RdhCRU::<V7>::load(&mut buf_reader).expect("Failed to load RdhCRUv7");
            let relative_offset =
                RelativeOffset::new((rdh_tmp.offset_to_next() as u64) - RDH_CRU_SIZE_BYTES);
            buf_reader
                .seek_relative(relative_offset.0)
                .expect("Error seeking");
            rdh_tmp
        })
        .collect();

    let filepath = std::path::PathBuf::from(fileout);
    let file = std::fs::File::options()
        .write(true)
        .create(true)
        .open(filepath)
        .unwrap();
    let mut buf_writer = std::io::BufWriter::new(file);
    rdhs.into_iter().for_each(|rdh| {
        buf_writer.write_all(rdh.to_byte_slice()).unwrap();
    });
}

pub fn bench_serialization_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization_write");

    let filename_manual = "manualrds.raw";

    group.bench_function("50k RDH writes", |b| {
        b.iter(|| write_rdh_manual(filename_manual))
    });
    // cleanup
    let filepath = std::path::PathBuf::from(filename_manual);
    std::fs::remove_file(filepath).unwrap();

    group.finish();
}

#[inline]
fn rdh0_deserialize(filename: &str, iterations: usize) {
    let filepath = std::path::PathBuf::from(filename);
    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(&filepath)
        .expect("File not found");
    let mut buf_reader = std::io::BufReader::new(file);
    for _i in 1..iterations {
        //println!("Iteration: {}", _i);
        let rdh0_tmp: Rdh0 = Rdh0::load(&mut buf_reader).expect("Failed to load Rdh0");
    }
}

#[inline]
fn rdh0_deserialize_no_macro(filename: &str, iterations: usize) {
    let filepath = std::path::PathBuf::from(filename);
    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(&filepath)
        .expect("File not found");
    let mut buf_reader = std::io::BufReader::new(file);
    for _i in 1..iterations {
        //println!("Iteration: {}", _i);
        let rdh0_tmp: Rdh0 = Rdh0::load_no_macro(&mut buf_reader).expect("Failed to load Rdh0");
    }
}

#[inline]
fn rdh0_deserialize_alternative_macro(filename: &str, iterations: usize) {
    let filepath = std::path::PathBuf::from(filename);
    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(&filepath)
        .expect("File not found");
    let mut buf_reader = std::io::BufReader::new(file);
    for _i in 1..iterations {
        //println!("Iteration: {}", _i);
        let rdh0_tmp: Rdh0 = Rdh0::load_alt_macro(&mut buf_reader).expect("Failed to load Rdh0");
    }
}

pub fn bench_rdh0_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("rdh0_deserialization");
    let filename = "../fastpasta_test_files/data_ols_ul.raw";
    for i in [1000, 10000, 50000, 100000, 500000, 1000000].iter() {
        group.bench_with_input(BenchmarkId::new("Current", i.to_string()), i, |b, i| {
            b.iter(|| rdh0_deserialize(black_box(filename), black_box(*i)))
        });
        group.bench_with_input(BenchmarkId::new("Alternative", i.to_string()), i, |b, i| {
            b.iter(|| rdh0_deserialize_alternative_macro(black_box(filename), black_box(*i)))
        });
        group.bench_with_input(BenchmarkId::new("No macro", i.to_string()), i, |b, i| {
            b.iter(|| rdh0_deserialize_no_macro(black_box(filename), black_box(*i)))
        });
    }
    group.finish();
}
