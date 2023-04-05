//! Contains the [Config] super trait, and all the sub traits required by it
//!
//! Implementing the [Config] super trait is required by configs passed to structs in other modules as part of instantiation.
use super::config::{Check, View};

/// Super trait for all the traits that needed to be implemented by the config struct
pub trait Config: Util + Filter + InputOutput + Checks + Views + Send + Sync {}

/// Trait for all small utility options that are not specific to any other trait
pub trait Util {
    /// Verbosity level of the logger: 0 = error, 1 = warn, 2 = info, 3 = debug, 4 = trace
    fn verbosity(&self) -> u8;
    /// Maximum number of errors to tolerate before exiting
    fn max_tolerate_errors(&self) -> u32;
}

/// Trait for all filter options
pub trait Filter {
    /// Link ID to filter by
    fn filter_link(&self) -> Option<u8>;
}

/// Trait for all input/output options
pub trait InputOutput {
    /// Input file to read from.
    fn input_file(&self) -> &Option<std::path::PathBuf>;
    /// Output file to write to.
    fn output(&self) -> &Option<std::path::PathBuf>;
    /// Output mode of the data writing (file, stdout, none)
    fn output_mode(&self) -> DataOutputMode;
}

/// Trait for all check options.
#[mockall::automock]
pub trait Checks {
    /// Type of Check to perform.
    fn check(&self) -> Option<Check>;
}

/// Trait for all view options.
pub trait Views {
    /// Type of View to generate.
    fn view(&self) -> Option<View>;
}

/// Enum for all possible data output modes.
#[derive(PartialEq)]
pub enum DataOutputMode {
    /// Write to a file.
    File,
    /// Write to stdout.
    Stdout,
    /// Do not write data out.
    None,
}
