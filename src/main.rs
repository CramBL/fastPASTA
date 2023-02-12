use fastpasta::data_words::rdh::RdhCRUv7;
use fastpasta::macros::print;
use fastpasta::{buf_reader_with_capacity, file_open_read_only, ByteSlice, GbtWord};

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
    const CAPACITY: usize = 1024 * 1024 * 10; // 10 MB
    let file = file_open_read_only(opt.files.first().unwrap())?;
    let mut buf_reader = buf_reader_with_capacity(file, CAPACITY);
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

    use std::{thread, time};

    let now = time::Instant::now();

    let mut rdhs = vec![rdh_cru, rdh_cru2];

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
        rdhs.push(tmp_rdh);
    }
    println!("Vec size: {}", rdhs.len());
    println!("example rdh-cru: {:?}", rdhs[40002]);
    println!("example rdh-cru: {:?}", rdhs[80002]);
    println!("example rdh-cru: {:?}", rdhs[45]);
    println!("Elapsed: {:?}", now.elapsed());
    //Write RDHs to file
    let filepath = PathBuf::from("rdhs.raw");
    let mut file = File::create(&filepath).unwrap();
    rdhs.into_iter().for_each(|rdh| {
        std::io::Write::write_all(&mut file, rdh.to_byte_slice()).unwrap();
    });
    println!("Elapsed: {:?}", now.elapsed());

    Ok(())
}
