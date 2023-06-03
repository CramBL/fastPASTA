use criterion::{criterion_group, criterion_main};

mod io_bench;
mod rdh_validation_bench;

criterion_group!(
    benches,
    io_bench::bench_rdh0_deserialization,
    // io_bench::bench_buffer_capacity,
    // io_bench::bench_deserialization,
    // io_bench::bench_serialization_write,
    // rdh_validation_bench::bench_rdh_sanity_check
);
criterion_main!(benches);
