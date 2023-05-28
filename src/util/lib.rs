//! Contains the [Config] super trait, and all the sub traits required by it
//!
//! Implementing the [Config] super trait is required by configs passed to structs in other modules as part of instantiation.

/// Re-export all the sub traits and enums
pub use super::config::{
    check::{CheckCommands, ChecksOpt, System, Target},
    filter::{FilterOpt, FilterTarget},
    inputoutput::{DataOutputMode, InputOutputOpt},
    util::UtilOpt,
    view::{ViewCommands, ViewOpt},
};

/// Super trait for all the traits that needed to be implemented by the config struct
// Generic traits that are required by the config struct
pub trait Config: Send + Sync + std::marker::Sized
where
    // Subtraits that group together related configuration options
    Self: UtilOpt + FilterOpt + InputOutputOpt + ChecksOpt + ViewOpt,
{
    /// Validate the arguments of the config
    fn validate_args(&self) -> Result<(), String> {
        if let Some(check) = self.check() {
            if let Some(target) = check.target() {
                if matches!(target, System::ITS_Stave) {
                    if self.filter_its_stave().is_none() {
                        return Err(
                            "Invalid config: Cannot check ITS stave without specifying a stave"
                                .to_string(),
                        );
                    }
                } else if self.check_its_trigger_period().is_some() {
                    return Err("Invalid config: Specifying trigger period has to be done with the `check all its_stave` command".to_string());
                }
            }
        }
        Ok(())
    }
}

impl<T> Config for &T
where
    T: Config,
{
    fn validate_args(&self) -> Result<(), String> {
        (*self).validate_args()
    }
}

impl<T> Config for Box<T>
where
    T: Config,
{
    fn validate_args(&self) -> Result<(), String> {
        (**self).validate_args()
    }
}
impl<T> Config for std::sync::Arc<T>
where
    T: Config,
{
    fn validate_args(&self) -> Result<(), String> {
        (**self).validate_args()
    }
}

#[allow(missing_docs)]
pub mod test_util {
    use super::*;
    use crate::util::config::{
        filter::FilterOpt,
        inputoutput::{DataOutputMode, InputOutputOpt},
    };
    #[derive(Debug, Clone)]

    /// Complete configurable Mock config for testing
    pub struct MockConfig {
        pub check: Option<CheckCommands>,
        pub view: Option<ViewCommands>,
        pub filter_link: Option<u8>,
        pub filter_fee: Option<u16>,
        pub filter_its_stave: Option<String>,
        pub verbosity: u8,
        pub max_tolerate_errors: u32,
        pub input_file: Option<std::path::PathBuf>,
        pub skip_payload: bool,
        pub output: Option<std::path::PathBuf>,
        pub output_mode: DataOutputMode,
        pub its_trigger_period: Option<u16>,
    }

    impl Default for MockConfig {
        fn default() -> Self {
            Self::new()
        }
    }

    impl MockConfig {
        pub fn new() -> Self {
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
                its_trigger_period: None,
            }
        }

        pub const fn const_default() -> Self {
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
                its_trigger_period: None,
            }
        }
    }

    impl Config for MockConfig {}
    impl ChecksOpt for MockConfig {
        fn check(&self) -> Option<CheckCommands> {
            self.check.clone()
        }
        fn check_its_trigger_period(&self) -> Option<u16> {
            self.its_trigger_period
        }
    }
    impl ViewOpt for MockConfig {
        fn view(&self) -> Option<ViewCommands> {
            self.view
        }
    }
    impl FilterOpt for MockConfig {
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
    impl UtilOpt for MockConfig {
        fn verbosity(&self) -> u8 {
            self.verbosity
        }

        fn max_tolerate_errors(&self) -> u32 {
            self.max_tolerate_errors
        }
    }
    impl InputOutputOpt for MockConfig {
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
