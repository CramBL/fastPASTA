use criterion::{black_box, BenchmarkId, Criterion};
use fastpasta_processing::input::prelude::*;

pub struct RelativeOffset(i64);
impl RelativeOffset {
    fn new(byte_offset: u64) -> Self {
        RelativeOffset(byte_offset as i64)
    }
}

// Sanity checking on RDHs
#[inline]
fn sanity_check_rdhs(rdh_cru_size_bytes: u64, filename: &str, iterations: usize) {
    let filepath = std::path::PathBuf::from(filename);
    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(&filepath)
        .expect("File not found");
    let mut buf_reader = std::io::BufReader::new(file);
    let mut rdh_validator =
        fastpasta_processing::analyze::validators::rdh::RdhCruSanityValidator::default();
    let mut rdhs = 0;

    loop {
        let tmp_rdh = match RdhCru::<V7>::load(&mut buf_reader) {
            Ok(rdh) => rdh,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                print!("EOF reached! ");
                break;
            }
            Err(e) => {
                println!("Error: {e}");
                break;
            }
        };
        let relative_offset =
            RelativeOffset::new((tmp_rdh.offset_to_next() as u64) - rdh_cru_size_bytes);
        buf_reader
            .seek_relative(relative_offset.0)
            .expect("Error seeking");
        match rdh_validator.sanity_check(&tmp_rdh) {
            Ok(_) => (),
            Err(e) => {
                println!("Error: {e}");
            }
        }
        rdhs += 1;
        if rdhs == iterations {
            break;
        }
    }
}

pub fn bench_rdh_sanity_check(c: &mut Criterion) {
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
