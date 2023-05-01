//! Contains the [Opt] struct that parses and stores the command line arguments
//!
//! [Opt] uses procedural macros from the [StructOpt] library to implement most of the argument parsing and validation logic.
//! The [Opt] struct implements several options and subcommands, as well as convenience functions to get various parts of the configuration

// Unfortunately needed because of the arg_enum macro not handling doc comments properly
#![allow(missing_docs)]
use super::lib::{Checks, Config, DataOutputMode, Filter, InputOutput, Util, Views};
use once_cell::sync::OnceCell;
use std::path::PathBuf;
use structopt::{clap::arg_enum, StructOpt};

pub static CONFIG: OnceCell<Cfg> = OnceCell::new();

/// The Opt struct uses the [StructOpt] procedural macros and implements the [Config] trait, to provide convenient access to the command line arguments.
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
pub struct Cfg {
    /// Input file (default: stdin)
    #[structopt(name = "INPUT DATA", parse(from_os_str))]
    file: Option<PathBuf>,

    /// Commands such as [Check] or [View] that accepts further subcommands
    #[structopt(subcommand)]
    cmd: Option<Command>,

    /// Verbosity level 0-4 (Errors, Warnings, Info, Debug, Trace)
    #[structopt(short = "v", long = "verbosity", default_value = "1", global = true)]
    verbosity: u8,

    /// Max tolerate errors before exiting, if set to 0 -> no limit to errors
    #[structopt(short = "e", long = "max-errors", default_value = "0", global = true)]
    max_tolerate_errors: u32,

    /// Set CRU link ID to filter by
    #[structopt(short = "f", long, global = true)]
    filter_link: Option<u8>,

    /// Output raw data (default: stdout), requires a link to filter by. If Checks or Views are enabled, the output is supressed.
    #[structopt(
        name = "OUTPUT DATA",
        short = "o",
        long = "output",
        parse(from_os_str),
        global = true,
        requires("filter-link")
    )]
    output: Option<PathBuf>,
}

impl Cfg {
    pub fn global() -> &'static Cfg {
        CONFIG.get().expect("Config is not initialized")
    }

    pub fn from_cli_args() -> Cfg {
        <Cfg as structopt::StructOpt>::from_args()
    }
}

/// Implementing the config super trait requires implementing all the sub traits
impl Config for Cfg {}

impl Views for Cfg {
    #[inline]
    fn view(&self) -> Option<View> {
        if let Some(sub_cmd) = &self.cmd {
            match sub_cmd {
                Command::View(view) => match view {
                    View::Rdh => Some(View::Rdh),
                    View::Hbf => Some(View::Hbf),
                },
                _ => None,
            }
        } else {
            None
        }
    }
}

impl Filter for Cfg {
    #[inline]
    fn filter_link(&self) -> Option<u8> {
        self.filter_link
    }
}

impl Checks for Cfg {
    #[inline]
    fn check(&self) -> Option<Check> {
        if let Some(sub_cmd) = &self.cmd {
            match sub_cmd {
                Command::Check(checks) => match checks {
                    Check::All(target) => Some(Check::All(target.clone())),
                    Check::Sanity(target) => Some(Check::Sanity(target.clone())),
                },
                Command::View(_) => None,
            }
        } else {
            None
        }
    }
}

impl InputOutput for Cfg {
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

    fn skip_payload(&self) -> bool {
        match (self.view(), self.check(), self.output_mode()) {
            // Skip payload in these cases
            (Some(View::Rdh), _, _) => true,
            (_, Some(Check::All(target)), _) | (_, Some(Check::Sanity(target)), _)
                if target.system.is_none() =>
            {
                true
            }
            // Don't skip payload in all other cases than above
            (_, _, _) => false,
        }
    }
}

impl Util for Cfg {
    #[inline]
    fn verbosity(&self) -> u8 {
        self.verbosity
    }
    #[inline]
    fn max_tolerate_errors(&self) -> u32 {
        self.max_tolerate_errors
    }
}

/// Possible subcommands at the upper level
#[derive(structopt::StructOpt, Debug, Clone)]
pub enum Command {
    /// [Check] subcommand to enable checks, needs to be followed by a [Check] type subcommand and a target system
    Check(Check),
    /// [View] subcommand to enable views, needs to be followed by a [View] type subcommand
    View(View),
}

/// Check subcommand to enable checks, needs to be followed by a check type subcommand and a target system
#[derive(structopt::StructOpt, Debug, Clone)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp, about = "Enable validation checks, by default only RDHs are checked.\n\
a target such as 'ITS' can be specified.\n\
Invoke `help [SUBCOMMAND]` for more information on possible targets.")]
pub enum Check {
    /// Perform sanity & running checks on RDH. If a target system is specified (e.g. 'ITS') checks implemented for the target is also performed. If no target system is specified, only the most generic checks are done.
    #[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
    All(Target),
    /// Perform only sanity checks on RDH. If a target system is specified (e.g. 'ITS') checks implemented for the target is also performed. If no target system is specified, only the most generic checks are done.
    #[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
    Sanity(Target),
}

impl Check {
    /// Get the target system for the check
    pub fn target(&self) -> Option<System> {
        match self {
            Check::All(target) => target.system.clone(),
            Check::Sanity(target) => target.system.clone(),
        }
    }
}

/// Data views that can be generated
#[derive(structopt::StructOpt, Debug, Clone)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp, about = "Enable data views")]
pub enum View {
    /// Print formatted RDHs to stdout
    #[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
    Rdh,
    /// Print formatted HBFs to stdout
    #[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
    Hbf,
}

/// Target system for checks
#[derive(structopt::StructOpt, Debug, Clone)]
pub struct Target {
    /// Target system for checks
    #[structopt(possible_values = &System::variants(), case_insensitive = true)]
    pub system: Option<System>,
}

arg_enum! {
/// List of supported systems to target for checks
#[derive(Debug, Clone)]
    pub enum System {
        ITS,
    }
}
