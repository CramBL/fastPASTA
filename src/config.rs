//! Contains the [Cfg] struct that parses and stores the command line arguments
//!
//! [Cfg] uses procedural macros from the `clap` library to implement most of the argument parsing and validation logic.
//! The [Cfg] struct implements several options and subcommands, as well as convenience functions to get various parts of the configuration

// Unfortunately needed because of the arg_enum macro not handling doc comments properly
#![allow(non_camel_case_types)]
use self::{
    check::{CheckCommands, ChecksOpt},
    custom_checks::{CustomChecks, CustomChecksOpt},
    filter::FilterOpt,
    inputoutput::{DataOutputMode, InputOutputOpt},
    prelude::Config,
    util::UtilOpt,
    view::{ViewCommands, ViewOpt},
};
use crate::words::its::layer_stave_string_to_feeid;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use std::sync::OnceLock;

pub mod check;
pub mod custom_checks;
pub mod filter;
pub mod inputoutput;
pub mod lib;
pub mod prelude;
pub mod test_util;
pub mod util;
pub mod view;
/// The [CONFIG] static variable is used to store the [Cfg] created from the parsed command line arguments
pub static CONFIG: OnceLock<Cfg> = OnceLock::new();
/// The [CUSTOM_CHECKS] static variable is used to store the [CustomChecks] created from the a TOML file specified through the parsed command line arguments
static CUSTOM_CHECKS: OnceLock<CustomChecks> = OnceLock::new();

/// The [Cfg] struct uses procedural macros and implements the [Config] trait, to provide convenient access to the command line arguments.
#[derive(Parser, Debug)]
#[command(name = "fastPASTA - fast Protocol Analysis Scanning Tool for ALICE")]
#[command(bin_name = "fastpasta", version)]
#[command(author = "Marc König <mbkj@tutamail.com>")]
#[command(about = "fastPASTA scans through ALICE Readout System's raw data output.")]
#[command(
    long_about = "\nfastpasta scans through ALICE Readout System's raw data output.\n\
It can report validation fails, display data in a human\n\
readable way, or filter the data.\n\
\n\
Project home page: https://gitlab.cern.ch/mkonig/fastpasta"
)]
#[command(propagate_version = true)]
pub struct Cfg {
    /// Input file (default: stdin)
    #[arg(name = "Raw Data", global = true)]
    file: Option<PathBuf>,

    /// Commands such as `Check` or `View` that accepts further subcommands
    #[command(subcommand)]
    cmd: Option<Command>,

    /// Verbosity level 0-4 (Errors, Warnings, Info, Debug, Trace)
    #[arg(short = 'v', long = "verbosity", default_value_t = 2, global = true)]
    verbosity: u8,

    /// Max tolerate errors before exiting, if set to 0 -> no limit to errors
    #[arg(
        short = 'e',
        long = "max-tolerate-errors",
        visible_aliases = ["max-errors", "tolerate-errors", "stop-at-error-count"],
        default_value_t = 0,
        global = true
    )]
    max_tolerate_errors: u32,

    /// Set the exit code for if any errors are detected in the input data (cannot be 0)
    #[arg(
        short = 'E',
        long = "any-errors-exit-code",
        visible_alias = "exit-code",
        global = true
    )]
    any_errors_exit_code: Option<u8>,

    /// Set CRU link ID to filter by (e.g. 5)
    #[arg(
        short = 'f',
        long,
        visible_alias = "link",
        global = true,
        group = "filter"
    )]
    filter_link: Option<u8>,

    /// Set FEE ID to filter by (e.g. 20522)
    #[arg(
        short = 'F',
        long,
        visible_alias = "fee",
        global = true,
        group = "filter"
    )]
    filter_fee: Option<u16>,

    /// Set ITS layer & stave to filter by (e.g. L5_42)
    #[arg(
        short = 's',
        long,
        name = "FILTER-ITS-STAVE",
        visible_aliases = ["its-stave", "stave"],
        global = true,
        group = "filter"
    )]
    filter_its_stave: Option<String>,

    /// Enables checks on the ITS trigger period with the specified value, usable with the `check all its-stave` command
    #[arg(short = 'p', long, global = true, requires = "FILTER-ITS-STAVE")]
    its_trigger_period: Option<u16>,

    /// Output raw data (default: stdout), requires setting a filter option. If Checks or Views are enabled, the output is supressed.
    #[arg(
        name = "OUTPUT DATA",
        short = 'o',
        long = "output",
        visible_alias = "out",
        global = true,
        requires("filter")
    )]
    output: Option<PathBuf>,

    /// Don't show error messages - helpful if there's a large amount of errors and you just want to see the report
    #[arg(short, long, default_value_t = false, global = true)]
    mute_errors: bool,

    /// Generate a check TOML file in the current directory that can be used as a template to configure checks against the raw data.
    #[arg(short, long, default_value_t = false, global = true, visible_aliases = ["gen-toml", "gen-checks"],)]
    generate_checks_toml: bool,

    /// Path to a checks TOML file that can be used to specify and customize certain checks against the raw data.
    #[arg(
        short = 'c',
        long,
        global = true,
        visible_aliases = ["custom-checks", "checks-file"],
      )]
    checks_toml: Option<PathBuf>,
}

impl Cfg {
    /// Get a reference to the global config
    pub fn global() -> &'static Cfg {
        CONFIG.get().expect("Config is not initialized")
    }

    /// If a checks TOML file is specified, parse it and set the custom checks static variable.
    /// If the checks TOML file is not specified, but the `--gen-checks-toml` flag is set, generate a checks TOML file in the current directory.
    pub fn handle_custom_checks(&self) {
        if let Some(checks_toml) = &self.checks_toml {
            let custom_checks = self.custom_checks_from_path(checks_toml);
            CUSTOM_CHECKS
                .set(custom_checks)
                .expect("Custom checks already initialized");
        } else if self.generate_custom_checks_toml_enabled() {
            self.generate_custom_checks_toml("custom_checks.toml");
        }
    }
}

/// Implementing the config super trait requires implementing all the sub traits
impl Config for Cfg {}

impl ViewOpt for Cfg {
    #[inline]
    fn view(&self) -> Option<ViewCommands> {
        if let Some(sub_cmd) = &self.cmd {
            match sub_cmd {
                Command::View(view_sub_cmd) => Some(view_sub_cmd.cmd),
                Command::Check(_) => None,
            }
        } else {
            None
        }
    }
}

impl FilterOpt for Cfg {
    #[inline]
    fn filter_link(&self) -> Option<u8> {
        self.filter_link
    }

    fn filter_fee(&self) -> Option<u16> {
        self.filter_fee
    }

    fn filter_its_stave(&self) -> Option<u16> {
        if let Some(stave_layer) = &self.filter_its_stave {
            // Start with something like "l2_1"
            // 1. check if the first char is an L, if so, it's the Lx_x format
            if stave_layer.to_uppercase().starts_with('L') {
                Some(layer_stave_string_to_feeid(stave_layer).expect("Invalid FEE ID"))
            } else {
                panic!("Invalid ITS layer & stave format, expected L[x]_[y], e.g. L2_13")
            }
        } else {
            None
        }
    }
}

impl ChecksOpt for Cfg {
    #[inline]
    fn check(&self) -> Option<CheckCommands> {
        if let Some(sub_cmd) = &self.cmd {
            match sub_cmd {
                Command::Check(checks) => match checks.cmd {
                    CheckCommands::All { system } => Some(CheckCommands::All { system }),
                    CheckCommands::Sanity { system } => Some(CheckCommands::Sanity { system }),
                },
                Command::View(_) => None,
            }
        } else {
            None
        }
    }

    fn check_its_trigger_period(&self) -> Option<u16> {
        self.its_trigger_period
    }
}

impl InputOutputOpt for Cfg {
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
            (Some(ViewCommands::Rdh), _, _) => true,
            (_, Some(CheckCommands::All { system: sys }), _)
            | (_, Some(CheckCommands::Sanity { system: sys }), _)
                if sys.is_none() =>
            {
                true
            }
            // Don't skip payload in all other cases than above
            (_, _, _) => false,
        }
    }
}

impl UtilOpt for Cfg {
    #[inline]
    fn verbosity(&self) -> u8 {
        self.verbosity
    }
    #[inline]
    fn max_tolerate_errors(&self) -> u32 {
        self.max_tolerate_errors
    }
    fn any_errors_exit_code(&self) -> Option<u8> {
        self.any_errors_exit_code
    }
    fn mute_errors(&self) -> bool {
        self.mute_errors
    }
}

impl CustomChecksOpt for Cfg {
    /// Get a reference to the [CustomChecks] struct, if it is initialized
    fn custom_checks(&self) -> Option<&'static CustomChecks> {
        CUSTOM_CHECKS.get()
    }

    fn custom_checks_enabled(&'static self) -> bool {
        self.custom_checks()
            .is_some_and(|c| *c != CustomChecks::default())
    }

    fn generate_custom_checks_toml_enabled(&self) -> bool {
        self.generate_checks_toml
    }

    fn cdps(&'static self) -> Option<u32> {
        if self.checks_toml.is_some() {
            self.custom_checks()
                .expect("Custom checks are not initialized")
                .cdps()
        } else {
            None
        }
    }

    fn triggers_pht(&'static self) -> Option<u32> {
        if self.checks_toml.is_some() {
            self.custom_checks()
                .expect("Custom checks are not initialized")
                .triggers_pht()
        } else {
            None
        }
    }

    fn rdh_version(&'static self) -> Option<u8> {
        if self.checks_toml.is_some() {
            self.custom_checks()
                .expect("Custom checks are not initialized")
                .rdh_version()
        } else {
            None
        }
    }

    fn chip_orders_ob(&'static self) -> Option<(&'static [u8], &'static [u8])> {
        if self.checks_toml.is_some() {
            self.custom_checks()
                .expect("Custom checks are not initialized")
                .chip_orders_ob()
        } else {
            None
        }
    }

    fn chip_count_ob(&'static self) -> Option<u8> {
        if self.checks_toml.is_some() {
            self.custom_checks()
                .expect("Custom checks are not initialized")
                .chip_count_ob()
        } else {
            None
        }
    }
}

/// Holds the [CheckCommands] subcommands
#[derive(Debug, Args, Clone, Copy)]
#[command(args_conflicts_with_subcommands = true)]
#[command(arg_required_else_help = true)]
pub struct CheckArgs {
    #[command(subcommand)]
    cmd: CheckCommands,
}
/// Holds the [ViewCommands] subcommands
#[derive(Debug, Args, Clone, Copy)]
#[command(args_conflicts_with_subcommands = true)]
#[command(arg_required_else_help = true)]
pub struct ViewArgs {
    #[command(subcommand)]
    cmd: ViewCommands,
}

#[derive(Debug, Subcommand, Clone, Copy)]
/// Subcommand to enable checks or views, needs to be followed by a [CheckCommands] (and optionally a target system) or [ViewCommands] subcommand.
pub enum Command {
    /// Subcommand to enable checks, needs to be followed by a [CheckCommands] type subcommand and a target system
    #[command(arg_required_else_help = true)]
    Check(CheckArgs),
    /// Subcommand to enable views, needs to be followed by a [ViewCommands] type subcommand
    #[command(arg_required_else_help = true)]
    View(ViewArgs),
}

impl CheckCommands {
    /// Get the target system for the check
    pub fn target(&self) -> Option<check::System> {
        match self {
            CheckCommands::All { system: sys } => *sys,
            CheckCommands::Sanity { system: sys } => *sys,
        }
    }
}

/// Get the [config][super::config::Cfg] from the command line arguments and set the static [CONFIG] variable.
pub fn init_config() -> Result<(), String> {
    let cfg = <super::config::Cfg as clap::Parser>::parse();
    cfg.validate_args()?;
    cfg.handle_custom_checks();
    crate::config::CONFIG.set(cfg).unwrap();
    Ok(())
}
