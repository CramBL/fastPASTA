//! Trait for all check options.
use clap::{Args, Subcommand, ValueEnum};
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
#[derive(Subcommand, Debug, Clone, Copy, PartialEq)]
pub enum CheckCommands {
    /// Perform sanity & running checks on RDH. If a target system is specified (e.g. 'ITS') checks implemented for the target is also performed. If no target system is specified, only the most generic checks are done.
    All {
        /// Optional target system for checks
        system: Option<System>,
    },
    /// Perform only sanity checks on RDH. If a target system is specified (e.g. 'ITS') checks implemented for the target is also performed. If no target system is specified, only the most generic checks are done.
    Sanity {
        /// Optional target system for checks
        system: Option<System>,
    },
}

/// Target system for checks
#[derive(Args, Debug, Clone, Copy, PartialEq)]
pub struct Target {
    /// Target system for checks
    pub system: Option<System>,
}

/// List of supported systems to target for checks
#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum System {
    /// Specify ITS as the target system for checks.
    ITS,
    /// Specify ITS stave as the target system for checks.
    ITS_Stave,
}
