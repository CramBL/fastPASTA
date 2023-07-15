//! Contains the [InputOutputOpt] Trait for all input/output options and the [DataOutputMode] enum for the output mode

use std::path::PathBuf;

/// Input/Output option set by a user
pub trait InputOutputOpt {
    /// Input file to read from.
    fn input_file(&self) -> &Option<PathBuf>;
    /// Determine from args if payload should be skipped at input
    fn skip_payload(&self) -> bool;
    /// Output file to write to.
    fn output(&self) -> &Option<PathBuf>;
    /// Output mode of the data writing (file, stdout, none)
    fn output_mode(&self) -> DataOutputMode;
}

impl<T> InputOutputOpt for &T
where
    T: InputOutputOpt,
{
    fn input_file(&self) -> &Option<PathBuf> {
        (*self).input_file()
    }
    fn skip_payload(&self) -> bool {
        (*self).skip_payload()
    }
    fn output(&self) -> &Option<PathBuf> {
        (*self).output()
    }
    fn output_mode(&self) -> DataOutputMode {
        (*self).output_mode()
    }
}

impl<T> InputOutputOpt for Box<T>
where
    T: InputOutputOpt,
{
    fn input_file(&self) -> &Option<PathBuf> {
        (**self).input_file()
    }
    fn skip_payload(&self) -> bool {
        (**self).skip_payload()
    }
    fn output(&self) -> &Option<PathBuf> {
        (**self).output()
    }
    fn output_mode(&self) -> DataOutputMode {
        (**self).output_mode()
    }
}
impl<T> InputOutputOpt for std::sync::Arc<T>
where
    T: InputOutputOpt,
{
    fn input_file(&self) -> &Option<PathBuf> {
        (**self).input_file()
    }
    fn skip_payload(&self) -> bool {
        (**self).skip_payload()
    }
    fn output(&self) -> &Option<PathBuf> {
        (**self).output()
    }
    fn output_mode(&self) -> DataOutputMode {
        (**self).output_mode()
    }
}

/// Enum for all possible data output modes.
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum DataOutputMode {
    /// Write to a file.
    File,
    /// Write to stdout.
    Stdout,
    /// Do not write data out.
    None,
}

impl std::fmt::Display for DataOutputMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataOutputMode::File => write!(f, "File"),
            DataOutputMode::Stdout => write!(f, "Stdout"),
            DataOutputMode::None => write!(f, "None"),
        }
    }
}
