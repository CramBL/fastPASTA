use std::path::PathBuf;
use structopt::StructOpt;
pub enum DataOutputMode {
    File,
    Stdout,
    None,
}
/// StructOpt is a library that allows parsing command line arguments
#[derive(StructOpt, Debug)]
#[structopt(
    name = "fastPASTA - fast Protocol Analysis Scanning Tool for ALICE",
    about = "\n\
Usage flow:  [INPUT] -> [FILTER] -> [VALIDATE] -> [OUTPUT]
                          ^^^          ^^^          ^^^
                        Optional     Optional     Optional
Examples:
    1. Read from file -> filter by link 0 -> validate -> output to file
        $ ./fastpasta input.raw --filter-link 0 --sanity-checks -o output.raw
    2. Read decompressed data from stdin -> filter link 3 & 4 -> pipe to validation checks
        $ lz4 -d input.raw | ./fastpasta --filter-link 3 4 | ./fastpasta --sanity-checks
                ^^^^                   ^^^^                           ^^^^
                INPUT     ->          FILTER              ->         VALIDATE"
)]
pub struct Opt {
    /// Input file (default: stdin)
    #[structopt(name = "INPUT DATA", parse(from_os_str))]
    file: Option<PathBuf>,

    /// Print formatted RDHs to stdout or file
    #[structopt(short, long)]
    print_rdhs: bool,

    /// Enable sanity checks
    #[structopt(short = "s", long = "sanity-checks")]
    sanity_checks: bool,

    /// Verbosity level 0-4 (Errors, Warnings, Info, Debug, Trace)
    #[structopt(short = "v", long = "verbosity", default_value = "0")]
    verbosity: u8,

    /// Max tolerate errors before exiting
    /// if set to 0 -> no limit to errors
    #[structopt(short = "e", long = "max-errors", default_value = "0")]
    max_tolerate_errors: u32,

    /// Set CRU link IDs to filter by
    #[structopt(short = "f", long)]
    filter_link: Option<Vec<u8>>,

    /// Output raw data (default: stdout)
    /// If checks are enabled, the output will be suppressed, unless this option is set explicitly
    #[structopt(name = "OUTPUT DATA", short = "o", long = "output", parse(from_os_str))]
    output: Option<PathBuf>,
}

impl Opt {
    #[inline]
    pub fn any_checks(&self) -> bool {
        self.sanity_checks()
    }
    #[inline]
    pub fn any_prints(&self) -> bool {
        self.print_rdhs()
    }
    #[inline]
    pub fn print_rdhs(&self) -> bool {
        self.print_rdhs
    }
    #[inline]
    pub fn sanity_checks(&self) -> bool {
        self.sanity_checks
    }
    #[inline]
    pub fn filter_link(&self) -> Option<Vec<u8>> {
        self.filter_link.clone()
    }
    #[inline]
    pub fn file(&self) -> &Option<PathBuf> {
        &self.file
    }
    #[inline]
    pub fn output(&self) -> &Option<PathBuf> {
        &self.output
    }
    #[inline]
    pub fn verbosity(&self) -> u8 {
        self.verbosity
    }
    #[inline]
    pub fn max_tolerate_errors(&self) -> u32 {
        self.max_tolerate_errors
    }

    // Determine data output mode
    #[inline]
    pub fn output_mode(&self) -> DataOutputMode {
        if self.output().is_some() {
            // if output is set to "stdout" output to stdout
            if self.output().as_ref().unwrap().to_str() == Some("stdout") {
                DataOutputMode::Stdout
            }
            // if output is set and a file path is given, output to file
            else {
                DataOutputMode::File
            }
        }
        // if output is not set, but checks or prints are enabled, suppress output
        else if self.any_checks() || self.any_prints() {
            DataOutputMode::None
        }
        // if output is not set and no checks are enabled, output to stdout
        else {
            DataOutputMode::Stdout
        }
    }

    #[inline]
    pub fn arg_validate(&self) -> Result<(), String> {
        let mut err_str = String::from("Invalid arguments: ");

        if let Some(filter_links) = &self.filter_link {
            if filter_links.is_empty() {
                err_str.push_str(" --filter-link must be followed by at least one link number");
            }
        }

        if err_str.len() > 20 {
            Err(err_str)
        } else {
            Ok(())
        }
    }
    #[inline]
    pub fn sort_link_args(&mut self) {
        if let Some(filter_links) = &mut self.filter_link {
            filter_links.sort();
        }
    }
}
