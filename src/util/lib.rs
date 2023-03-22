pub trait Config: Util + Filter + InputOutput + Checks + Views + Send + Sync {}

pub trait Util {
    fn verbosity(&self) -> u8;
    fn max_tolerate_errors(&self) -> u32;
}

pub trait Filter {
    fn filter_link(&self) -> Option<u8>;
}

pub trait InputOutput {
    fn input_file(&self) -> &Option<std::path::PathBuf>;
    fn output(&self) -> &Option<std::path::PathBuf>;
    fn output_mode(&self) -> DataOutputMode;
}
#[mockall::automock]
pub trait Checks {
    fn check(&self) -> Option<Check>;
}

pub trait Views {
    fn view(&self) -> Option<View>;
}

pub enum DataOutputMode {
    File,
    Stdout,
    None,
}

#[derive(structopt::StructOpt, Debug, Clone)]
pub enum Command {
    Check(Check),
    View(View),
}

#[derive(structopt::StructOpt, Debug, Clone)]
pub enum Check {
    All(Target),
    Sanity(Target),
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
pub enum View {
    /// Print formatted RDHs to stdout or file
    Rdh,
}

#[derive(structopt::StructOpt, Debug, Clone)]
pub struct Target {
    #[structopt(subcommand)]
    pub target: Option<Data>,
}

#[derive(structopt::StructOpt, Debug, Clone)]
pub enum Data {
    Rdh,
}
