//! Contains the [ErrorStats] struct which stores error messages observed in the raw data and related data
use crate::util::*;

type LayerStave = (u8, u8);

/// Stores error messages observed during analysis
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorStats {
    fatal_error: Option<Box<str>>,
    reported_errors: Vec<Box<str>>,
    custom_checks_stats_errors: Vec<Box<str>>,
    total_errors: u64,
    unique_error_codes: Option<Vec<String>>,
    // Only applicable if the data is from ITS
    staves_with_errors: Option<Vec<LayerStave>>,
}

impl ErrorStats {
    /// If data processing is done, sort error messages, extract unique error codes etc.
    pub(super) fn finalize_stats(
        &mut self,
        mute_errors: bool,
        layer_staves_seen: Option<&[(u8, u8)]>,
    ) {
        if !mute_errors {
            // Sort stats by memory position where they were found before consuming them
            self.sort_error_msgs_by_mem_pos();
        }
        if let Some(layer_staves_seen) = layer_staves_seen {
            self.check_errors_for_stave_id(layer_staves_seen);
        }
        // Extract unique error codes from reported errors
        self.process_unique_error_codes();
    }

    pub(super) fn sort_error_msgs_by_mem_pos(&mut self) {
        // Regex to extract the memory address from the error message
        // State machine: https://regexper.com/#0x%28%5B0-9A-F%5D%2B%29%3A
        let re = Regex::new(r"^0x(?<mem_pos>[0-9A-F]+)").unwrap();
        // Sort the errors by memory address
        if !self.reported_errors.is_empty() {
            self.reported_errors.sort_unstable_by_key(|e| {
                let addr = re
                    .captures(e)
                    .unwrap_or_else(|| panic!("Error parsing memory address from error msg: {e}"));
                u64::from_str_radix(&addr["mem_pos"], 16).expect("Error parsing memory address")
            });
        }
    }

    pub(super) fn process_unique_error_codes(&mut self) {
        // Extract unique error codes from reported errors
        if !self.reported_errors.is_empty() {
            let unique_error_codes = extract_unique_error_codes(&self.reported_errors);
            self.unique_error_codes = Some(unique_error_codes);
        }

        // If there's any errors from the custom checks on stats, find the error codes and add them.
        if !self.custom_checks_stats_errors.is_empty() {
            let unique_custom_error_codes: Vec<String> =
                extract_unique_error_codes(&self.custom_checks_stats_errors);
            if self.unique_error_codes.is_none() {
                self.unique_error_codes = Some(unique_custom_error_codes);
            } else {
                self.unique_error_codes
                    .as_mut()
                    .unwrap()
                    .extend(unique_custom_error_codes);
                self.unique_error_codes.as_mut().unwrap().dedup();
            }
        }
    }

    pub(super) fn check_errors_for_stave_id(&mut self, layer_staves_seen: &[LayerStave]) {
        if self.staves_with_errors.is_some() {
            return;
        }
        let mut staves_with_errors: Vec<LayerStave> = Vec::new();

        // State machine:
        // https://regexper.com/#FEE%28%3F%3A.%7C%29ID%3A%28%5B1-9%5D%5B0-9%5D%7B0%2C4%7D%29
        let re: Regex = Regex::new("FEE(?:.|)ID:(?P<fee_id>[1-9][0-9]{0,4})").unwrap();

        self.reported_errors.iter().for_each(|err_msg| {
            let fee_id_match: Option<regex::Captures> = re.captures(err_msg);
            if let Some(id_capture) = fee_id_match {
                let fee_id = id_capture["fee_id"].parse::<u16>().unwrap();
                let layer = layer_from_feeid(fee_id);
                let stave = stave_number_from_feeid(fee_id);

                let stave_with_error = layer_staves_seen
                    .iter()
                    .find(|(l, s)| *l == layer && *s == stave)
                    .expect(
                        "FEE ID found in error message that does not match any layer/stave seen",
                    );

                if !staves_with_errors.contains(stave_with_error) {
                    staves_with_errors.push(*stave_with_error);
                }
            }
        });
        self.staves_with_errors = Some(staves_with_errors);
    }

    pub(super) fn err_count(&self) -> u64 {
        self.total_errors
    }

    pub(super) fn add_err(&mut self, error_msg: Box<str>) {
        self.total_errors += 1;
        self.reported_errors.push(error_msg);
    }

    pub(super) fn add_custom_check_error(&mut self, error_msg: Box<str>) {
        self.total_errors += 1;
        self.custom_checks_stats_errors.push(error_msg);
    }

    pub(super) fn add_fatal_err(&mut self, error_msg: Box<str>) {
        self.fatal_error = Some(error_msg);
    }

    pub(super) fn any_fatal_err(&self) -> bool {
        self.fatal_error.is_some()
    }

    pub(super) fn fatal_err(&self) -> &str {
        self.fatal_error.as_ref().unwrap()
    }

    pub(super) fn unique_error_codes_as_slice(&self) -> &[String] {
        self.unique_error_codes
            .as_ref()
            .expect("Unique error codes were never set")
    }

    /// Returns a slice of Layer/Staves seen in error messages
    ///
    /// Returns ´None´ if no staves were seen in any error messages
    /// Panics if staves with errors is None when there's errors.
    pub(crate) fn staves_with_errors_as_slice(&self) -> Option<&[LayerStave]> {
        if self.staves_with_errors.is_none() {
            if self.total_errors == 0 {
                return None;
            } else {
                panic!("Staves with errors were never set")
            }
        }
        Some(self.staves_with_errors.as_ref().unwrap())
    }

    /// Return an iterator over all error messages
    pub fn errors_as_slice_iter(&self) -> impl Iterator<Item = &Box<str>> {
        self.reported_errors
            .iter()
            .chain(self.fatal_error.iter())
            .chain(self.custom_checks_stats_errors.iter())
    }

    pub(super) fn validate_other(&self, other: &Self) -> Result<(), Vec<String>> {
        // This syntax is used to ensure that a compile error is raised if a new field is added to the struct but not added to the validation here
        // Also add it to the `validate_fields` macro!
        let other = Self {
            fatal_error: other.fatal_error.clone(),
            reported_errors: other.reported_errors.clone(),
            custom_checks_stats_errors: other.custom_checks_stats_errors.clone(),
            total_errors: other.total_errors,
            unique_error_codes: other.unique_error_codes.clone(),
            staves_with_errors: other.staves_with_errors.clone(),
        };

        self.validate_fields(&other)
    }

    crate::validate_fields!(
        ErrorStats,
        fatal_error,
        reported_errors,
        custom_checks_stats_errors,
        total_errors,
        unique_error_codes,
        staves_with_errors
    );
}

fn extract_unique_error_codes(error_messages: &[Box<str>]) -> Vec<String> {
    let mut error_codes: Vec<String> = Vec::new();
    let re = Regex::new(r"\[E(?P<err_code>[0-9]{2,4})\]").unwrap();
    error_messages.iter().for_each(|err_msg| {
        let err_code_matches: Vec<String> = re
            .captures_iter(err_msg)
            .map(|m| m.name("err_code").unwrap().as_str().into())
            .collect();

        err_code_matches.into_iter().for_each(|err_code| {
            if !error_codes.contains(&err_code) {
                error_codes.push(err_code);
            }
        });
    });
    error_codes
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_serde_consistency() {
        // Test JSON and TOML serialization/deserialization
        let mut error_stats = ErrorStats::default();

        error_stats.add_err("0xE0: [E0001] Error message".into());
        error_stats.finalize_stats(false, None);

        let error_stats_ser_json = serde_json::to_string(&error_stats).unwrap();
        let error_stats_de_json: ErrorStats = serde_json::from_str(&error_stats_ser_json).unwrap();
        assert_eq!(error_stats, error_stats_de_json);
        println!("{}", serde_json::to_string_pretty(&error_stats).unwrap());

        let error_stats_ser_toml = toml::to_string(&error_stats).unwrap();
        let error_stats_de_toml: ErrorStats = toml::from_str(&error_stats_ser_toml).unwrap();
        assert_eq!(error_stats, error_stats_de_toml);
        println!("{}", error_stats_ser_toml);
    }
}
