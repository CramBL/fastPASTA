use alice_protocol_reader::prelude::*;
use criterion::Criterion;
use std::io::Write;

pub struct RelativeOffset(i64);
impl RelativeOffset {
    fn new(byte_offset: u64) -> Self {
        RelativeOffset(byte_offset as i64)
    }
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
    let rdhs: Vec<RdhCru<V7>> = (0..50000)
        .map(|_| {
            let rdh_tmp = RdhCru::<V7>::load(&mut buf_reader).expect("Failed to load RdhCruv7");
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
