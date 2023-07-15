//! Contains the Trait [ViewOpt] for all view options, and the [ViewCommands] enum for the view mode

use clap::Subcommand;
/// Data views that can be generated
#[derive(Subcommand, Copy, Clone, Debug, PartialEq, Eq)]
pub enum ViewCommands {
    /// Print formatted RDHs to stdout
    Rdh,
    /// DEPRECATED! use its-readout-frames instead. Print formatted ITS payload HBFs to stdout, validating the printed words with a Protocol Tracker.
    Hbf,
    /// Print formatted ITS readout frames to stdout
    ItsReadoutFrames,
    /// Print formatted ITS readout frames with Data Words to stdout
    ItsReadoutFramesData,
}

/// Trait for all view options set by the user.
pub trait ViewOpt {
    /// Type of View to generate.
    fn view(&self) -> Option<ViewCommands>;
}

impl<T> ViewOpt for &T
where
    T: ViewOpt,
{
    fn view(&self) -> Option<ViewCommands> {
        (*self).view()
    }
}

impl<T> ViewOpt for Box<T>
where
    T: ViewOpt,
{
    fn view(&self) -> Option<ViewCommands> {
        (**self).view()
    }
}

impl<T> ViewOpt for std::sync::Arc<T>
where
    T: ViewOpt,
{
    fn view(&self) -> Option<ViewCommands> {
        (**self).view()
    }
}
