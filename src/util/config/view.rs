//! Contains the Trait [ViewOpt] for all view options, and the [View] enum for the view mode

/// Trait for all view options set by the user.
pub trait ViewOpt {
    /// Type of View to generate.
    fn view(&self) -> Option<View>;
}

impl<T> ViewOpt for &T
where
    T: ViewOpt,
{
    fn view(&self) -> Option<View> {
        (*self).view()
    }
}

impl<T> ViewOpt for Box<T>
where
    T: ViewOpt,
{
    fn view(&self) -> Option<View> {
        (**self).view()
    }
}

impl<T> ViewOpt for std::sync::Arc<T>
where
    T: ViewOpt,
{
    fn view(&self) -> Option<View> {
        (**self).view()
    }
}

/// Data views that can be generated
#[derive(structopt::StructOpt, Debug, Clone)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
pub enum View {
    /// Print formatted RDHs to stdout
    #[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
    Rdh,
    /// DEPRECATED! use its-readout-frames instead. Print formatted ITS payload HBFs to stdout, validating the printed words with a Protocol Tracker.
    #[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
    Hbf,
    /// Print formatted ITS readout frames to stdout
    #[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
    ItsReadoutFrames,
}
