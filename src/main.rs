use fastpasta::data_words::rdh::RdhCRUv7;
use fastpasta::{buf_reader_with_capacity, file_open_read_only, ByteSlice, GbtWord};

use std::path::PathBuf;

use structopt::StructOpt;
/// StructOpt is a library that allows parsing command line arguments
#[derive(StructOpt, Debug)]
#[structopt(
    name = "fastPASTA - fast Protocol Analysis Scanning Tool for ALICE",
    about = "A tool to scan and verify the CRU protocol of the ALICE readout system"
)]
struct Opt {
    /// Dump RDHs to stdout or file
    #[structopt(short, long = "dump-rhds")]
    dump_rhds: bool,

    /// Activate sanity checks
    #[structopt(short = "s", long = "sanity-checks")]
    sanity_checks: bool,

    /// Files to process
    #[structopt(name = "FILE", parse(from_os_str))]
    file: PathBuf,

    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,
}

const RDH_CRU_SIZE_BYTES: u64 = 64;

pub struct RelativeOffset(i64);
pub enum SeekError {
    EOF,
}

impl RelativeOffset {
    fn new(byte_offset: u64) -> Self {
        RelativeOffset(byte_offset as i64)
    }
}

pub fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();
    println!("{:#?}", opt);
    const CAPACITY: usize = 1024 * 10; // 10 KB
    let file = file_open_read_only(&opt.file)?;
    let mut buf_reader = buf_reader_with_capacity(file, CAPACITY);
    use std::time;

    let now = time::Instant::now();

    let mut rdhs: Vec<RdhCRUv7> = vec![];

    let rdh_validator = fastpasta::validators::rdh::RDH_CRU_V7_VALIDATOR;
    let mut processed = 0;

    let mut running_rdh_checker = fastpasta::validators::rdh::RdhCruv7RunningChecker::new();

    loop {
        let tmp_rdh = match RdhCRUv7::load(&mut buf_reader) {
            Ok(rdh) => rdh,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                print!("EOF reached! ");
                println!("{} packets processed", processed);
                break;
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        };
        let relative_offset =
            RelativeOffset::new((tmp_rdh.offset_new_packet as u64) - RDH_CRU_SIZE_BYTES);
        buf_reader
            .seek_relative(relative_offset.0)
            .expect("Error seeking");

        if opt.sanity_checks {
            match rdh_validator.sanity_check(&tmp_rdh) {
                Ok(_) => (),
                Err(e) => {
                    println!("Sanity check failed: {}", e);
                    break;
                }
            }
        }

        // RDH CHECH: There is always page 0 + minimum page 1 + stop flag
        match running_rdh_checker.check(&tmp_rdh) {
            Ok(_) => (),
            Err(e) => {
                println!("RDH check failed: {}", e);
                println!("Last RDH: {:?}", running_rdh_checker.last_rdh2.unwrap());
                println!("Current RDH: {:?}", tmp_rdh);
                println!("Processed: {}", processed);

                break;
            }
        }

        processed += 1;
        if opt.dump_rhds {
            if opt.output.is_some() {
                rdhs.push(tmp_rdh);
            } else {
                println!("{:?}", tmp_rdh);
            }
        }
    }
    println!("Vec size: {}", rdhs.len());
    println!("Elapsed: {:?}", now.elapsed());
    //Write RDHs to file
    if opt.output.is_some() {
        let filepath = PathBuf::from(opt.output.unwrap());
        let mut file = std::fs::File::create(&filepath).unwrap();
        rdhs.into_iter().for_each(|rdh| {
            std::io::Write::write_all(&mut file, rdh.to_byte_slice()).unwrap();
        });
        println!("Elapsed: {:?}", now.elapsed());
    }

    debug_assert!(running_rdh_checker.expect_pages_counter == 0);

    Ok(())
}
