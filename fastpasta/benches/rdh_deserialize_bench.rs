use alice_protocol_reader::prelude::*;
use criterion::{black_box, BenchmarkId, Criterion};
const BENCH_FILE_PATH: &str = "/home/mkonig/rawdata-debugging/data_ols_v7.raw";

#[inline]
fn deserialize_rdh_current(filename: &str, iterations: usize) {
    let filepath = std::path::PathBuf::from(filename);
    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(filepath)
        .expect("File not found");
    let mut buf_reader = std::io::BufReader::new(file);
    for _i in 1..iterations {
        let _rdh_tmp: RdhCru = SerdeRdh::load(&mut buf_reader).expect("Failed to load RdhCruv7");
    }
}

pub fn bench_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("deserialization");
    let filename = BENCH_FILE_PATH;
    for i in [1_000, 10_000, 100_000, 1_000_000].iter() {
        group.bench_with_input(BenchmarkId::new("current", i.to_string()), i, |b, i| {
            b.iter(|| deserialize_rdh_current(black_box(filename), black_box(*i)))
        });
        // group.bench_with_input(BenchmarkId::new("alternative", i.to_string()), i, |b, i| {
        //     b.iter(|| deserialize_rdh_alt(black_box(filename), black_box(*i)))
        // });
    }
    group.finish();
}

#[inline]
fn rdh0_deserialize(filename: &str, iterations: usize) {
    let filepath = std::path::PathBuf::from(filename);
    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(filepath)
        .expect("File not found");
    let mut buf_reader = std::io::BufReader::new(file);
    for _i in 1..iterations {
        //println!("Iteration: {}", _i);
        let _rdh0_tmp: Rdh0 = Rdh0::load(&mut buf_reader).expect("Failed to load Rdh0");
    }
}

#[inline]
fn _rdh0_deserialize_alternative(filename: &str, iterations: usize) {
    let filepath = std::path::PathBuf::from(filename);
    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(filepath)
        .expect("File not found");
    let mut _buf_reader = std::io::BufReader::new(file);
    for _i in 1..iterations {
        //println!("Iteration: {}", _i);
        // let _rdh0_tmp: Rdh0 = Rdh0::load_alt(&mut buf_reader).expect("Failed to load Rdh0");
    }
}

pub fn bench_rdh0_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("rdh0_deserialization");
    let filename = BENCH_FILE_PATH;
    for i in [1000, 10000, 50000, 100000, 500000, 1000000].iter() {
        group.bench_with_input(BenchmarkId::new("Current", i.to_string()), i, |b, i| {
            b.iter(|| rdh0_deserialize(black_box(filename), black_box(*i)))
        });
        // group.bench_with_input(BenchmarkId::new("Alternative", i.to_string()), i, |b, i| {
        //     b.iter(|| rdh0_deserialize_alternative(black_box(filename), black_box(*i)))
        // });
    }
    group.finish();
}
