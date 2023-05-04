//! Contains the [Config] super trait, and all the sub traits required by it
//!
//! Implementing the [Config] super trait is required by configs passed to structs in other modules as part of instantiation.
use std::fmt::Display;

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
    /// FEE ID to filter by
    fn filter_fee(&self) -> Option<u16>;
    /// ITS layer & stave to filter by
    fn filter_its_stave(&self) -> Option<u16>;

    /// Get the target of the filter
    fn filter_target(&self) -> Option<FilterTarget> {
        #[allow(clippy::manual_map)] // Clippy is wrong here
        if let Some(link) = self.filter_link() {
            Some(FilterTarget::Link(link))
        } else if let Some(fee) = self.filter_fee() {
            Some(FilterTarget::Fee(fee))
        } else if let Some(its_layer_stave) = self.filter_its_stave() {
            Some(FilterTarget::ItsLayerStave(its_layer_stave))
        } else {
            None
        }
    }

    /// Determine if the filter is enabled
    fn filter_enabled(&self) -> bool {
        self.filter_link().is_some()
            || self.filter_fee().is_some()
            || self.filter_its_stave().is_some()
    }
}

#[derive(Debug, Clone, Copy)]
/// The target of an optional filter on the input data
pub enum FilterTarget {
    /// Filter on the link ID
    Link(u8),
    /// Filter on the FEE ID
    Fee(u16),
    /// Filter on the ITS layer and stave
    ItsLayerStave(u16),
}

/// Trait for all input/output options
pub trait InputOutput {
    /// Input file to read from.
    fn input_file(&self) -> &Option<std::path::PathBuf>;
    /// Determine from args if payload should be skipped at input
    fn skip_payload(&self) -> bool;
    /// Output file to write to.
    fn output(&self) -> &Option<std::path::PathBuf>;
    /// Output mode of the data writing (file, stdout, none)
    fn output_mode(&self) -> DataOutputMode;
}

/// Trait for all check options.
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
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum DataOutputMode {
    /// Write to a file.
    File,
    /// Write to stdout.
    Stdout,
    /// Do not write data out.
    None,
}

impl Display for DataOutputMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataOutputMode::File => write!(f, "File"),
            DataOutputMode::Stdout => write!(f, "Stdout"),
            DataOutputMode::None => write!(f, "None"),
        }
    }
}

#[allow(missing_docs)]
pub mod test_util {
    use super::*;
    #[derive(Debug, Clone)]

    /// Complete configurable Mock config for testing
    pub struct MockConfig {
        pub check: Option<Check>,
        pub view: Option<View>,
        pub filter_link: Option<u8>,
        pub filter_fee: Option<u16>,
        pub filter_its_stave: Option<String>,
        pub verbosity: u8,
        pub max_tolerate_errors: u32,
        pub input_file: Option<std::path::PathBuf>,
        pub skip_payload: bool,
        pub output: Option<std::path::PathBuf>,
        pub output_mode: DataOutputMode,
    }

    impl Default for MockConfig {
        fn default() -> Self {
            Self {
                check: None,
                view: None,
                filter_link: None,
                filter_fee: None,
                filter_its_stave: None,
                verbosity: 0,
                max_tolerate_errors: 0,
                input_file: None,
                skip_payload: false,
                output: None,
                output_mode: DataOutputMode::None,
            }
        }
    }

    impl Config for MockConfig {}
    impl Checks for MockConfig {
        fn check(&self) -> Option<Check> {
            self.check.clone()
        }
    }
    impl Views for MockConfig {
        fn view(&self) -> Option<View> {
            self.view.clone()
        }
    }
    impl Filter for MockConfig {
        fn filter_link(&self) -> Option<u8> {
            self.filter_link
        }

        fn filter_fee(&self) -> Option<u16> {
            self.filter_fee
        }

        fn filter_its_stave(&self) -> Option<u16> {
            if let Some(stave_layer) = &self.filter_its_stave {
                // Start with something like "l2_1"
                // 1. check if the first char is an L, if so, it's the Lx_x format
                if stave_layer.to_uppercase().starts_with('L') {
                    Some(
                        crate::words::its::layer_stave_string_to_feeid(stave_layer)
                            .expect("Invalid FEE ID"),
                    )
                } else {
                    panic!("Invalid ITS layer & stave format, expected L[layer numer]_[stave number], e.g. L2_1, got {stave_layer}")
                }
            } else {
                None
            }
        }
    }
    impl Util for MockConfig {
        fn verbosity(&self) -> u8 {
            self.verbosity
        }

        fn max_tolerate_errors(&self) -> u32 {
            self.max_tolerate_errors
        }
    }
    impl InputOutput for MockConfig {
        fn input_file(&self) -> &Option<std::path::PathBuf> {
            &self.input_file
        }

        fn skip_payload(&self) -> bool {
            self.skip_payload
        }

        fn output(&self) -> &Option<std::path::PathBuf> {
            &self.output
        }

        fn output_mode(&self) -> DataOutputMode {
            self.output_mode
        }
    }
}
