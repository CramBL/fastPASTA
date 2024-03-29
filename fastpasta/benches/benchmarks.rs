use criterion::{criterion_group, criterion_main};

mod io_bench;
mod rdh_deserialize_bench;
mod rdh_serialize_bench;
mod rdh_validation_bench;
mod trigger_stats;

criterion_group!(
    benches,
    rdh_deserialize_bench::bench_rdh0_deserialization,
    io_bench::bench_buffer_capacity,
    rdh_deserialize_bench::bench_deserialization,
    rdh_serialize_bench::bench_serialization_write,
    rdh_validation_bench::bench_rdh_sanity_check,
    trigger_stats::bench_collect_trigger_stats,
);
criterion_main!(benches);
