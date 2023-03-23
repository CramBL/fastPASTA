use super::config::{Check, View};

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

#[derive(PartialEq)]
pub enum DataOutputMode {
    File,
    Stdout,
    None,
}
