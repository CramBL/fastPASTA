//! Contains the [InputOutputOpt] Trait for all input/output options and the [DataOutputMode] enum for the output mode

use crate::util::*;

/// Input/Output option set by a user
pub trait InputOutputOpt {
    /// Input file to read from.
    fn input_file(&self) -> Option<&Path>;
    /// Output file to write to.
    fn output(&self) -> Option<&Path>;
    /// Output mode of the data writing (file, stdout, none)
    fn output_mode(&self) -> DataOutputMode;
    /// Stats output mode (file, stdout, none)
    fn stats_output_mode(&self) -> DataOutputMode;
    /// Stats output format (JSON, TOML)
    fn stats_output_format(&self) -> Option<DataOutputFormat>;
    /// Input stats file to read from and verify match with collected stats at end of analysis.
    fn input_stats_file(&self) -> Option<&Path>;
}

impl<T> InputOutputOpt for &T
where
    T: InputOutputOpt,
{
    fn input_file(&self) -> Option<&Path> {
        (*self).input_file()
    }
    fn output(&self) -> Option<&Path> {
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
    fn input_stats_file(&self) -> Option<&Path> {
        (*self).input_stats_file()
    }
}

impl<T> InputOutputOpt for Box<T>
where
    T: InputOutputOpt,
{
    fn input_file(&self) -> Option<&Path> {
        (**self).input_file()
    }
    fn output(&self) -> Option<&Path> {
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
    fn input_stats_file(&self) -> Option<&Path> {
        (**self).input_stats_file()
    }
}
impl<T> InputOutputOpt for Arc<T>
where
    T: InputOutputOpt,
{
    fn input_file(&self) -> Option<&Path> {
        (**self).input_file()
    }
    fn output(&self) -> Option<&Path> {
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
    fn input_stats_file(&self) -> Option<&Path> {
        (**self).input_stats_file()
    }
}

/// Enum for all possible data output modes.
#[derive(PartialEq, Debug, Clone)]
pub enum DataOutputMode {
    /// Write to a file.
    File(Box<Path>),
    /// Write to stdout.
    Stdout,
    /// Do not write data out.
    None,
}

impl fmt::Display for DataOutputMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataOutputMode::File(p) => write!(f, "File({})", p.display()),
            DataOutputMode::Stdout => write!(f, "Stdout"),
            DataOutputMode::None => write!(f, "None"),
        }
    }
}

impl FromStr for DataOutputMode {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "STDOUT" => Ok(DataOutputMode::Stdout),
            "NONE" => Ok(DataOutputMode::None),
            _ => Ok(DataOutputMode::File(Path::new(s).into())),
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

impl fmt::Display for DataOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataOutputFormat::JSON => write!(f, "JSON"),
            DataOutputFormat::TOML => write!(f, "TOML"),
        }
    }
}

impl FromStr for DataOutputFormat {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "JSON" => Ok(DataOutputFormat::JSON),
            "TOML" => Ok(DataOutputFormat::TOML),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid data output format",
            )),
        }
    }
}
