//! Contains the [Config] super trait, and all the sub traits required by it
//!
//! Implementing the [Config] super trait is required by configs passed to structs in other modules as part of instantiation.

use std::sync::{atomic::AtomicBool, Arc};

/// Re-export all the sub traits and enums
pub use super::config::{
    check::{CheckCommands, ChecksOpt, System, Target},
    custom_checks::CustomChecksOpt,
    filter::{FilterOpt, FilterTarget},
    inputoutput::{DataOutputMode, InputOutputOpt},
    util::UtilOpt,
    view::{ViewCommands, ViewOpt},
    Cfg,
};

/// Super trait for all the traits that needed to be implemented by the config struct
// Generic traits that are required by the config struct
pub trait Config: Send + Sync + std::marker::Sized
where
    // Subtraits that group together related configuration options
    Self: UtilOpt + FilterOpt + InputOutputOpt + ChecksOpt + ViewOpt + CustomChecksOpt,
{
    /// Validate the arguments of the config
    fn validate_args(&self) -> Result<(), String> {
        if let Some(check) = self.check() {
            if let Some(target) = check.target() {
                if matches!(check, CheckCommands::Sanity { system } if matches!(system, Some(System::ITS_Stave)))
                {
                    return Err("Invalid config: Cannot check ITS stave with `check sanity`, instead use `check all its-stave`".to_string());
                }
                if !matches!(target, System::ITS_Stave) && self.check_its_trigger_period().is_some()
                {
                    return Err("Invalid config: Specifying trigger period has to be done with the `check all its-stave` command".to_string());
                }
            }
        }
        if self.any_errors_exit_code().is_some_and(|val| val == 0) {
            return Err("Invalid config: Exit code for any errors cannot be 0".to_string());
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

/// Start the [stderrlog] instance, and immediately use it to log the configured [DataOutputMode].
pub fn init_error_logger(cfg: &(impl UtilOpt + InputOutputOpt)) {
    stderrlog::new()
        .module("fastpasta")
        .verbosity(cfg.verbosity() as usize)
        .init()
        .expect("Failed to initialize logger");
    match cfg.output_mode() {
        DataOutputMode::Stdout => log::trace!("Data ouput set to stdout"),
        DataOutputMode::File => log::trace!("Data ouput set to file"),
        DataOutputMode::None => {
            log::trace!("Data output set to suppressed")
        }
    }
    log::trace!("Starting fastpasta with args: {:#?}", Cfg::global());
    log::trace!("Checks enabled: {:#?}", Cfg::global().check());
    log::trace!("Views enabled: {:#?}", Cfg::global().view());
}

/// Get the [config][super::config::Cfg] from the command line arguments and set the static [CONFIG][crate::util::config::CONFIG] variable.
pub fn init_config() -> Result<(), String> {
    let cfg = <super::config::Cfg as clap::Parser>::parse();
    cfg.validate_args()?;
    cfg.handle_custom_checks();
    crate::util::config::CONFIG.set(cfg).unwrap();
    Ok(())
}

/// Initializes the Ctrl+C handler to facilitate graceful shutdown on Ctrl+C
///
/// Also handles SIGTERM and SIGHUP if the `termination` feature is enabled
pub fn init_ctrlc_handler(stop_flag: Arc<AtomicBool>) {
    // Handles SIGINT, SIGTERM and SIGHUP (as the `termination` feature is  enabled)
    ctrlc::set_handler({
        let mut stop_sig_count = 0;
        move || {
            log::warn!(
                "Stop Ctrl+C, SIGTERM, or SIGHUP received, stopping gracefully, please wait..."
            );
            stop_flag.store(true, std::sync::atomic::Ordering::SeqCst);
            stop_sig_count += 1;
            if stop_sig_count > 1 {
                log::warn!("Second stop signal received, ungraceful shutdown.");
                std::process::exit(1);
            }
        }
    })
    .expect("Error setting Ctrl-C handler");
}

/// Exits the program with the appropriate exit code
pub fn exit(exit_code: u8, any_errors_flag: Arc<AtomicBool>) -> std::process::ExitCode {
    if exit_code == 0 {
        log::info!("Exit successful from data processing");

        if Cfg::global().any_errors_exit_code().is_some()
            && any_errors_flag.load(std::sync::atomic::Ordering::Relaxed)
        {
            std::process::ExitCode::from(Cfg::global().any_errors_exit_code().unwrap())
        } else {
            std::process::ExitCode::SUCCESS
        }
    } else {
        std::process::ExitCode::from(exit_code)
    }
}

#[allow(missing_docs)]
pub mod test_util {
    use super::*;
    use crate::util::config::{
        custom_checks::CustomChecks,
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
        pub exit_code_any_errors: Option<u8>,
        pub mute_errors: bool,
        pub generate_checks_toml: bool,
        pub custom_checks: Option<CustomChecks>,
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
                exit_code_any_errors: None,
                mute_errors: false,
                generate_checks_toml: false,
                custom_checks: None,
            }
        }
    }

    impl Config for MockConfig {}

    impl ChecksOpt for MockConfig {
        fn check(&self) -> Option<CheckCommands> {
            self.check
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

        fn any_errors_exit_code(&self) -> Option<u8> {
            self.exit_code_any_errors
        }

        fn mute_errors(&self) -> bool {
            self.mute_errors
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

    impl CustomChecksOpt for MockConfig {
        fn generate_custom_checks_toml_enabled(&self) -> bool {
            self.generate_checks_toml
        }

        fn cdps(&self) -> Option<u32> {
            if self.custom_checks.is_some() {
                self.custom_checks.as_ref().unwrap().cdps()
            } else {
                None
            }
        }

        fn triggers_pht(&self) -> Option<u32> {
            if self.custom_checks.is_some() {
                self.custom_checks.as_ref().unwrap().triggers_pht()
            } else {
                None
            }
        }
    }
}
