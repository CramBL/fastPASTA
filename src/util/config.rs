use std::path::PathBuf;
use structopt::StructOpt;

use super::lib::{Checks, Config, DataOutputMode, Filter, InputOutput, Util, Views};
/// StructOpt is a library that allows parsing command line arguments
#[derive(StructOpt, Debug)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp,
    name = "fastPASTA - fast Protocol Analysis Scanning Tool for ALICE",
    about = "\n\
Usage flow:  [INPUT] -> [FILTER] -> [VALIDATE/VIEW/OUTPUT]
                          ^^^                 ^^^
                        Optional            Optional
Examples:
    1. Read from file -> filter by link 0 -> validate with all checks enabled
        $ ./fastpasta input.raw --filter-link 0 check all
    2. Read decompressed data from stdin -> filter link 3 -> see a formatted view of RDHs
        $ lz4 -d input.raw -c | ./fastpasta --filter-link 3 | ./fastpasta view rdh
                 ^^^^                      ^^^^                       ^^^^
                INPUT       --->          FILTER          --->        VIEW"
)]
pub struct Opt {
    /// Input file (default: stdin)
    #[structopt(name = "INPUT DATA", parse(from_os_str))]
    file: Option<PathBuf>,

    #[structopt(subcommand)]
    cmd: Option<Command>,

    /// Verbosity level 0-4 (Errors, Warnings, Info, Debug, Trace)
    #[structopt(short = "v", long = "verbosity", default_value = "0")]
    verbosity: u8,

    /// Max tolerate errors before exiting,
    /// if set to 0 -> no limit to errors
    #[structopt(short = "e", long = "max-errors", default_value = "0")]
    max_tolerate_errors: u32,

    /// Set CRU link ID to filter by
    #[structopt(short = "f", long)]
    filter_link: Option<u8>,

    /// Output raw data (default: stdout)
    /// If checks are enabled, the output will be suppressed, unless this option is set explicitly
    #[structopt(name = "OUTPUT DATA", short = "o", long = "output", parse(from_os_str))]
    output: Option<PathBuf>,
}
impl Config for Opt {}

impl Views for Opt {
    #[inline]
    fn view(&self) -> Option<View> {
        if let Some(sub_cmd) = &self.cmd {
            match sub_cmd {
                Command::View(view) => match view {
                    View::Rdh => Some(View::Rdh),
                },
                _ => None,
            }
        } else {
            None
        }
    }
}

impl Filter for Opt {
    #[inline]
    fn filter_link(&self) -> Option<u8> {
        self.filter_link
    }
}

impl Checks for Opt {
    #[inline]
    fn check(&self) -> Option<Check> {
        if let Some(sub_cmd) = &self.cmd {
            match sub_cmd {
                Command::Check(checks) => match checks {
                    Check::All(target) => Some(Check::All(target.clone())),
                    Check::Sanity(target) => Some(Check::Sanity(target.clone())),
                    Check::Running(target) => Some(Check::Running(target.clone())),
                },
                Command::View(_) => None,
            }
        } else {
            None
        }
    }
}

impl InputOutput for Opt {
    #[inline]
    fn input_file(&self) -> &Option<PathBuf> {
        &self.file
    }
    #[inline]
    fn output(&self) -> &Option<PathBuf> {
        &self.output
    }
    // Determine data output mode
    #[inline]
    fn output_mode(&self) -> DataOutputMode {
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
        else if self.check().is_some() || self.view().is_some() {
            DataOutputMode::None
        }
        // if output is not set and no checks are enabled, output to stdout
        else {
            DataOutputMode::Stdout
        }
    }
}

impl Util for Opt {
    #[inline]
    fn verbosity(&self) -> u8 {
        self.verbosity
    }
    #[inline]
    fn max_tolerate_errors(&self) -> u32 {
        self.max_tolerate_errors
    }
}

#[derive(structopt::StructOpt, Debug, Clone)]
pub enum Command {
    Check(Check),
    View(View),
}

#[derive(structopt::StructOpt, Debug, Clone)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp, about = "Enable validation checks, a target for the checks can be specified.\n\
If no target is specified, all data will be checked.\n\
Invoke `help [SUBCOMMAND]` for more information on possible targets.")]
pub enum Check {
    /// Perform all checks on target data. If no target is specified, all data will be checked.
    #[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
    All(Target),
    /// Perform sanity checks on target data. If no target is specified, all data will be checked.
    #[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
    Sanity(Target),
    /// Perform sanity & running checks on target data. If no target is specified, all data will be checked.
    #[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
    Running(Target),
}

impl Check {
    pub fn target(&self) -> Option<Data> {
        match self {
            Check::All(target) => target.target.clone(),
            Check::Sanity(target) => target.target.clone(),
            Check::Running(target) => target.target.clone(),
        }
    }
}

#[derive(structopt::StructOpt, Debug, Clone)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp, about = "Enable data views")]
pub enum View {
    /// Print formatted RDHs to stdout
    #[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
    Rdh,
}

#[derive(structopt::StructOpt, Debug, Clone)]
pub struct Target {
    #[structopt(subcommand)]
    pub target: Option<Data>,
}

#[derive(structopt::StructOpt, Debug, Clone)]
pub enum Data {
    /// Target RDHs with all enabled checks.
    #[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
    Rdh,
}
