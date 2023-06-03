use criterion::{black_box, BenchmarkId, Criterion};
use fastpasta::words::{
    lib::{RdhSubWord, SerdeRdh, RDH_CRU},
    rdh::Rdh0,
    rdh_cru::{RdhCRU, V7},
};

pub struct RelativeOffset(i64);
impl RelativeOffset {
    fn new(byte_offset: u64) -> Self {
        RelativeOffset(byte_offset as i64)
    }
}

#[inline]
fn deserialize_rdh_current(filename: &str, iterations: usize) {
    let filepath = std::path::PathBuf::from(filename);
    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(&filepath)
        .expect("File not found");
    let mut buf_reader = std::io::BufReader::new(file);
    for _i in 1..iterations {
        let rdh_tmp: RdhCRU<V7> = SerdeRdh::load(&mut buf_reader).expect("Failed to load RdhCRUv7");
    }
}

#[inline]
fn deserialize_rdh_alt(filename: &str, iterations: usize) {
    let filepath = std::path::PathBuf::from(filename);
    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(&filepath)
        .expect("File not found");
    let mut buf_reader = std::io::BufReader::new(file);
    for _i in 1..iterations {
        let rdh_tmp: RdhCRU<V7> =
            SerdeRdh::load_alt(&mut buf_reader).expect("Failed to load RdhCRUv7");
    }
}

#[inline]
fn deserialize_rdh_buf(filename: &str, iterations: usize) {
    let filepath = std::path::PathBuf::from(filename);
    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(&filepath)
        .expect("File not found");
    let mut buf_reader = std::io::BufReader::new(file);
    for _i in 1..iterations {
        let rdh_tmp: RdhCRU<V7> =
            SerdeRdh::load_buf(&mut buf_reader).expect("Failed to load RdhCRUv7");
    }
}

pub fn bench_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("deserialization");
    let filename = "../fastpasta_test_files/data_ols_ul.raw";
    for i in [1_000, 10_000, 100_000, 1_000_000].iter() {
        group.bench_with_input(BenchmarkId::new("current", i.to_string()), i, |b, i| {
            b.iter(|| deserialize_rdh_current(black_box(filename), black_box(*i)))
        });
        group.bench_with_input(BenchmarkId::new("alternative", i.to_string()), i, |b, i| {
            b.iter(|| deserialize_rdh_alt(black_box(filename), black_box(*i)))
        });
        group.bench_with_input(BenchmarkId::new("from_buf", i.to_string()), i, |b, i| {
            b.iter(|| deserialize_rdh_buf(black_box(filename), black_box(*i)))
        });
    }
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
        let _rdh0_tmp: Rdh0 = Rdh0::load(&mut buf_reader).expect("Failed to load Rdh0");
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
        let _rdh0_tmp: Rdh0 = Rdh0::load_no_macro(&mut buf_reader).expect("Failed to load Rdh0");
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
        let _rdh0_tmp: Rdh0 = Rdh0::load_alt(&mut buf_reader).expect("Failed to load Rdh0");
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
