use fastpasta::data_words::rdh::RdhCRUv7;
use fastpasta::GbtWord;
use std::io::Read;
use std::path::PathBuf;
use std::{fs::File, io::Seek};

use structopt::StructOpt;
/// StructOpt is a library that allows parsing command line arguments
#[derive(StructOpt, Debug)]
#[structopt(
    name = "fastPASTA - fast Protocol Analysis Scanning Tool for ALICE",
    about = "A tool to scan and verify the CRU protocol of the ALICE readout system"
)]
struct Opt {
    /// Files to process
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,

    /// Number of bytes to read
    #[structopt(short, long, default_value = "10")]
    bytes: usize,
}

const RDH_CRU_SIZE_BYTES: u64 = 64;

struct RelativeOffset(i64);
pub enum SeekError {
    EOF,
}

impl RelativeOffset {
    fn new(byte_offset: u64) -> Self {
        RelativeOffset(byte_offset as i64)
    }
    fn next(byte_offset: u64, f_len: u64) -> Result<RelativeOffset, SeekError> {
        if byte_offset >= f_len {
            return Err(SeekError::EOF);
        }
        Ok(RelativeOffset(byte_offset as i64))
    }
}

pub fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();
    println!("{:#?}", opt);
    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(opt.files.first().unwrap())
        .expect("File not found");

    let mut buf_reader = std::io::BufReader::new(file);
    let rdh_cru = RdhCRUv7::load(&mut buf_reader).expect("Error loading RDH");
    // Size of an RDH needs to be subtracted from the offset_new_packet seek to the right position
    // it is possible to move the file cursor, but this is not recommended as it requires a mutable file descriptor
    let relative_offset =
        RelativeOffset::new((rdh_cru.offset_new_packet as u64) - RDH_CRU_SIZE_BYTES);
    buf_reader
        .seek_relative(relative_offset.0)
        .expect("Error seeking");
    let rdh_cru2 = RdhCRUv7::load(&mut buf_reader).expect("Error loading RDH");
    let relative_offset =
        RelativeOffset::new((rdh_cru2.offset_new_packet as u64) - RDH_CRU_SIZE_BYTES);
    rdh_cru.print();
    rdh_cru2.print();
    buf_reader
        .seek_relative(relative_offset.0)
        .expect("Error seeking");

    for i in 1..500000 {
        let tmp_rdh = match RdhCRUv7::load(&mut buf_reader) {
            Ok(rdh) => rdh,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                print!("EOF reached! ");
                println!("{} packets processed", i);
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
        if tmp_rdh.rdh0.header_id != 7 {
            println!("WRONG header ID: {}", tmp_rdh.rdh0.header_id);
        }
        if i % 100 == 0 {
            println!("{} packets processed", i);
        }
        if i == 40000 {
            println!("40000 packets processed");
            print!("RDH 40000: ");
            tmp_rdh.print();
        }
    }
    Ok(())
}
