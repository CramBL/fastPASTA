#![allow(missing_docs)] // Necessary as the arg_enum macro doesn't allow comments
//! Trait for all check options.
/// A config that implements this trait can be used to enable checks.
pub trait ChecksOpt {
    /// Type of Check to perform.
    fn check(&self) -> Option<Check>;

    /// Return the check on ITS trigger period if it is set.
    fn check_its_trigger_period(&self) -> Option<u16>;
}

impl<T> ChecksOpt for &T
where
    T: ChecksOpt,
{
    fn check(&self) -> Option<Check> {
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
    fn check(&self) -> Option<Check> {
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
    fn check(&self) -> Option<Check> {
        (**self).check()
    }
    fn check_its_trigger_period(&self) -> Option<u16> {
        (**self).check_its_trigger_period()
    }
}

/// Check subcommand to enable checks, needs to be followed by a check type subcommand and a target system
#[derive(structopt::StructOpt, Debug, Clone)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
pub enum Check {
    /// Perform sanity & running checks on RDH. If a target system is specified (e.g. 'ITS') checks implemented for the target is also performed. If no target system is specified, only the most generic checks are done.
    #[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
    All(Target),
    /// Perform only sanity checks on RDH. If a target system is specified (e.g. 'ITS') checks implemented for the target is also performed. If no target system is specified, only the most generic checks are done.
    #[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
    Sanity(Target),
}

/// Target system for checks
#[derive(structopt::StructOpt, Debug, Clone)]
pub struct Target {
    /// Target system for checks
    #[structopt(possible_values = &System::variants(), case_insensitive = true)]
    pub system: Option<System>,
}

use structopt::clap::arg_enum;
arg_enum! {
/// List of supported systems to target for checks
#[derive(Debug, Clone, PartialEq)]
    pub enum System {
        ITS,
        ITS_Stave,
    }
}
