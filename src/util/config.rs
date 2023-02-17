use std::path::PathBuf;
use structopt::StructOpt;
/// StructOpt is a library that allows parsing command line arguments
#[derive(StructOpt, Debug)]
#[structopt(
    name = "fastPASTA - fast Protocol Analysis Scanning Tool for ALICE",
    about = "A tool to scan and verify the CRU protocol of the ALICE readout system"
)]
pub struct Opt {
    /// Dump RDHs to stdout or file
    #[structopt(short, long = "dump-rhds")]
    dump_rhds: bool,

    /// Activate sanity checks
    #[structopt(short = "s", long = "sanity-checks")]
    sanity_checks: bool,

    /// links to filter
    #[structopt(short = "f", long)]
    filter_link: Option<u8>,

    /// File to process
    #[structopt(name = "FILE", parse(from_os_str))]
    file: PathBuf,

    /// Output file
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,
}

impl Opt {
    pub fn dump_rhds(&self) -> bool {
        self.dump_rhds
    }
    pub fn sanity_checks(&self) -> bool {
        self.sanity_checks
    }
    pub fn filter_link(&self) -> Option<u8> {
        self.filter_link
    }
    pub fn file(&self) -> &PathBuf {
        &self.file
    }
    pub fn output(&self) -> &Option<PathBuf> {
        &self.output
    }
}
