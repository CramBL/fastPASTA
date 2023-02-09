use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use fastpasta::{GbtWord, RdhCRUv7};
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

pub fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();
    println!("{:#?}", opt);
    let offset: i64 = 0;
    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(opt.files.first().unwrap())
        .expect("File not found");
    let mut buf_reader = std::io::BufReader::new(file);
    let rdh_cru = RdhCRUv7::load(&mut buf_reader);
    let offset = offset.wrapping_add_unsigned(rdh_cru.offset_new_packet as u64);
    buf_reader.seek_relative(offset).expect("Error seeking");
    let rdh_cru2 = RdhCRUv7::load(&mut buf_reader);
    rdh_cru.print();
    rdh_cru2.print();
    Ok(())
}

/// Parse and print the data of the files with the given number of bytes to read from the CLI
///
/// Iterates through the files provided on the CLI and reads into a buffer consisting of a vector of bytes
/// Reads the number of bytes specified on the CLI, if that number exceeds the length of the file, it will read the entire file
/// Finally prints the bytes in hex format, efficiently by slicing the vector
///
/// # Examples
/// ```
/// let files = vec!["file1.txt", "file2.txt"];
/// let bytes_to_read = 10;
/// parse_and_print_data_files(files, bytes_to_read);
/// ```
/// # Example output on 2 files with 10 bytes to read:
/// ```
/// ../Downloads/data_ols_ul.raw contains: [7, 40, 2a, 50, 0, 20, 0, 0, e0, 13]
/// ../Downloads/data_ols_no_ul.raw contains: [7, 40, 2a, 50, 0, 20, 0, 0, b0, 1f]
/// ```
pub fn parse_and_print_data_files(files_in: Vec<PathBuf>, bytes_to_read: usize) {
    files_in.iter().for_each(|file| {
        let count = 1000;
        let mut f = File::open(file.to_owned()).expect("File not found");
        let mut buf: Vec<u8> = vec![0; count];
        f.read_exact(&mut buf).expect("Error reading file");
        let bytes_to_print = match buf.len() {
            b_len if b_len < bytes_to_read => b_len,
            _ => bytes_to_read,
        };
        println!(
            "{filepath} contains: {data:x?}",
            data = &buf[0..bytes_to_print],
            filepath = file.display(),
        );
    });
}
