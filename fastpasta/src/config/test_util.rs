#![allow(missing_docs)]

use self::custom_checks::custom_checks_cfg::CustomChecks;
use crate::util::*;
use custom_checks::CustomChecksOpt;

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
    pub input_file: Option<PathBuf>,
    pub skip_payload: bool,
    pub output: Option<PathBuf>,
    pub output_mode: DataOutputMode,
    pub its_trigger_period: Option<u16>,
    pub exit_code_any_errors: Option<u8>,
    pub mute_errors: bool,
    pub generate_checks_toml: bool,
    pub custom_checks: Option<CustomChecks>,
    pub stats_output_mode: DataOutputMode,
    pub stats_output_format: Option<DataOutputFormat>,
    pub stats_input_file: Option<PathBuf>,
    pub show_error_codes: Vec<String>,
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
            stats_output_mode: DataOutputMode::None,
            stats_output_format: None,
            stats_input_file: None,
            show_error_codes: Vec::new(),
        }
    }

    pub fn new_check_all_its() -> Self {
        Self {
            check: Some(CheckCommands::All(CheckModeArgs {
                target: Some(System::ITS),
                ..Default::default()
            })),
            ..Default::default()
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
    fn skip_payload(&self) -> bool {
        self.skip_payload
    }

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

    fn error_code_filter(&self) -> Option<&[String]> {
        if self.show_error_codes.is_empty() {
            None
        } else {
            Some(&self.show_error_codes)
        }
    }

    fn disable_styled_views(&self) -> bool {
        true
    }
}
impl InputOutputOpt for MockConfig {
    fn input_file(&self) -> Option<&Path> {
        self.input_file.as_deref()
    }

    fn output(&self) -> Option<&Path> {
        self.output.as_deref()
    }

    fn output_mode(&self) -> DataOutputMode {
        self.output_mode.clone()
    }

    fn stats_output_mode(&self) -> DataOutputMode {
        self.stats_output_mode.clone()
    }

    fn stats_output_format(&self) -> Option<super::inputoutput::DataOutputFormat> {
        self.stats_output_format
    }

    fn input_stats_file(&self) -> Option<&Path> {
        self.stats_input_file.as_deref()
    }
}

impl CustomChecksOpt for MockConfig {
    fn custom_checks(&'static self) -> Option<&'static CustomChecks> {
        self.custom_checks.as_ref()
    }

    fn custom_checks_enabled(&'static self) -> bool {
        self.custom_checks()
            .is_some_and(|c| *c != CustomChecks::default())
    }

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

    fn rdh_version(&self) -> Option<u8> {
        if self.custom_checks.is_some() {
            self.custom_checks.as_ref().unwrap().rdh_version()
        } else {
            None
        }
    }

    fn chip_orders_ob(&'static self) -> Option<&[Vec<u8>]> {
        if self.custom_checks.is_some() {
            self.custom_checks.as_ref().unwrap().chip_orders_ob()
        } else {
            None
        }
    }

    fn chip_count_ob(&'static self) -> Option<u8> {
        if self.custom_checks.is_some() {
            self.custom_checks.as_ref().unwrap().chip_count_ob()
        } else {
            None
        }
    }
}
