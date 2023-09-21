//! Contains the [InputOutputOpt] Trait for all input/output options and the [DataOutputMode] enum for the output mode

use std::path::PathBuf;

/// Input/Output option set by a user
pub trait InputOutputOpt {
    /// Input file to read from.
    fn input_file(&self) -> Option<&PathBuf>;
    /// Output file to write to.
    fn output(&self) -> Option<&PathBuf>;
    /// Output mode of the data writing (file, stdout, none)
    fn output_mode(&self) -> DataOutputMode;
    /// Stats output mode (file, stdout, none)
    fn stats_output_mode(&self) -> DataOutputMode;
    /// Stats output format (JSON, TOML)
    fn stats_output_format(&self) -> Option<DataOutputFormat>;
    /// Input stats file to read from and verify match with collected stats at end of analysis.
    fn input_stats_file(&self) -> Option<&PathBuf>;
}

impl<T> InputOutputOpt for &T
where
    T: InputOutputOpt,
{
    fn input_file(&self) -> Option<&PathBuf> {
        (*self).input_file()
    }
    fn output(&self) -> Option<&PathBuf> {
        (*self).output()
    }
    fn output_mode(&self) -> DataOutputMode {
        (*self).output_mode()
    }
    fn stats_output_mode(&self) -> DataOutputMode {
        (*self).stats_output_mode()
    }
    fn stats_output_format(&self) -> Option<DataOutputFormat> {
        (*self).stats_output_format()
    }
    fn input_stats_file(&self) -> Option<&PathBuf> {
        (*self).input_stats_file()
    }
}

impl<T> InputOutputOpt for Box<T>
where
    T: InputOutputOpt,
{
    fn input_file(&self) -> Option<&PathBuf> {
        (**self).input_file()
    }
    fn output(&self) -> Option<&PathBuf> {
        (**self).output()
    }
    fn output_mode(&self) -> DataOutputMode {
        (**self).output_mode()
    }
    fn stats_output_mode(&self) -> DataOutputMode {
        (**self).stats_output_mode()
    }
    fn stats_output_format(&self) -> Option<DataOutputFormat> {
        (**self).stats_output_format()
    }
    fn input_stats_file(&self) -> Option<&PathBuf> {
        (**self).input_stats_file()
    }
}
impl<T> InputOutputOpt for std::sync::Arc<T>
where
    T: InputOutputOpt,
{
    fn input_file(&self) -> Option<&PathBuf> {
        (**self).input_file()
    }
    fn output(&self) -> Option<&PathBuf> {
        (**self).output()
    }
    fn output_mode(&self) -> DataOutputMode {
        (**self).output_mode()
    }
    fn stats_output_mode(&self) -> DataOutputMode {
        (**self).stats_output_mode()
    }
    fn stats_output_format(&self) -> Option<DataOutputFormat> {
        (**self).stats_output_format()
    }
    fn input_stats_file(&self) -> Option<&PathBuf> {
        (**self).input_stats_file()
    }
}

/// Enum for all possible data output modes.
#[derive(PartialEq, Debug, Clone)]
pub enum DataOutputMode {
    /// Write to a file.
    File(Box<std::path::Path>),
    /// Write to stdout.
    Stdout,
    /// Do not write data out.
    None,
}

impl std::fmt::Display for DataOutputMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataOutputMode::File(p) => write!(f, "File({})", p.display()),
            DataOutputMode::Stdout => write!(f, "Stdout"),
            DataOutputMode::None => write!(f, "None"),
        }
    }
}

impl std::str::FromStr for DataOutputMode {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "STDOUT" => Ok(DataOutputMode::Stdout),
            "NONE" => Ok(DataOutputMode::None),
            _ => Ok(DataOutputMode::File(std::path::Path::new(s).into())),
        }
    }
}

/// Enum for all possible data output formats.
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum DataOutputFormat {
    /// JSON format.
    JSON,
    /// TOML format.
    TOML,
}

impl std::fmt::Display for DataOutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataOutputFormat::JSON => write!(f, "JSON"),
            DataOutputFormat::TOML => write!(f, "TOML"),
        }
    }
}

impl std::str::FromStr for DataOutputFormat {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "JSON" => Ok(DataOutputFormat::JSON),
            "TOML" => Ok(DataOutputFormat::TOML),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid data output format",
            )),
        }
    }
}
