//! Trait for all check options.
use std::path::PathBuf;

use clap::{Args, Subcommand};
/// A config that implements this trait can be used to enable checks.
pub trait ChecksOpt {
    /// Type of Check to perform.
    fn check(&self) -> Option<CheckCommands>;

    /// Return the check on ITS trigger period if it is set.
    fn check_its_trigger_period(&self) -> Option<u16>;
}

impl<T> ChecksOpt for &T
where
    T: ChecksOpt,
{
    fn check(&self) -> Option<CheckCommands> {
        (*self).check()
    }
    fn check_its_trigger_period(&self) -> Option<u16> {
        (*self).check_its_trigger_period()
    }
}

impl<T> ChecksOpt for Box<T>
where
    T: ChecksOpt,
{
    fn check(&self) -> Option<CheckCommands> {
        (**self).check()
    }
    fn check_its_trigger_period(&self) -> Option<u16> {
        (**self).check_its_trigger_period()
    }
}
impl<T> ChecksOpt for std::sync::Arc<T>
where
    T: ChecksOpt,
{
    fn check(&self) -> Option<CheckCommands> {
        (**self).check()
    }
    fn check_its_trigger_period(&self) -> Option<u16> {
        (**self).check_its_trigger_period()
    }
}

/// Check subcommand to enable checks, needs to be followed by a check type subcommand and a target system
#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum CheckCommands {
    /// Perform sanity & running checks on RDH. If a target system is specified (e.g. 'ITS') checks implemented for the target is also performed. If no target system is specified, only the most generic checks are done.
    All(CheckModeArgs),
    /// Perform only sanity checks on RDH. If a target system is specified (e.g. 'ITS') checks implemented for the target is also performed. If no target system is specified, only the most generic checks are done.
    Sanity(CheckModeArgs),
}

#[derive(Debug, Args, Clone, PartialEq, Default)]
#[command(args_conflicts_with_subcommands = true)]
/// Arguments to All/Sanity check modes, default values are [None].
pub struct CheckModeArgs {
    #[command(subcommand)]
    /// A target system to enable checks for
    pub target: Option<System>,

    /// Placeholder allowing not specifying a system and instead just supplying the path to raw data following the check mode argument
    ///
    /// e.g. `check all <rawdata_file.raw>`
    #[command(flatten)]
    pub path: CmdPathArg,
}

#[derive(Debug, Default, Args, Clone, PartialEq)]
/// The placeholder value for supplying a path to raw data in the position where a target system would otherwise be specified
pub struct CmdPathArg {
    #[arg(short = 'r', long)]
    path: Option<PathBuf>,
}

/// List of supported systems to target for checks
#[derive(Subcommand, Copy, Clone, Debug, PartialEq, Eq)]
pub enum System {
    /// Specify ITS as the target system for checks.
    ITS,
    /// Specify ITS stave as the target system for checks.
    ITS_Stave,
}
