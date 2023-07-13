use crate::words::its::{layer_from_feeid, stave_number_from_feeid};

type LayerStave = (u8, u8);

#[derive(Default, Debug, Clone)]
pub struct ErrorStats {
    fatal_error: Option<String>,
    reported_errors: Vec<String>,
    custom_checks_stats_errors: Vec<String>,
    total_errors: u64,
    unique_error_codes: Option<Vec<u16>>,
    // Only applicable if the data is from ITS
    staves_with_errors: Option<Vec<LayerStave>>,
}

impl ErrorStats {
    /// If data processing is done, sort error messages, extract unique error codes etc.
    pub(super) fn finalize_stats(&mut self) {
        // Extract unique error codes from reported errors
        self.process_unique_error_codes();
    }

    pub(super) fn sort_error_msgs_by_mem_pos(&mut self) {
        // Regex to extract the memory address from the error message
        let re = regex::Regex::new(r"0x(?P<mem_pos>[0-9a-fA-F]+):").unwrap();
        // Sort the errors by memory address
        if !self.reported_errors.is_empty() {
            self.reported_errors.sort_by_key(|e| {
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

        // If there's any errors from the custom checks on stats, and the error codes doesn't already contain `E99` then add it.
        if !self.custom_checks_stats_errors.is_empty() {
            if self.unique_error_codes.is_none() {
                self.unique_error_codes = Some(vec![99]);
            } else if self
                .unique_error_codes
                .as_deref()
                .is_some_and(|err_codes| !err_codes.contains(&99))
            {
                self.unique_error_codes.as_mut().unwrap().push(99);
            }
        }
    }

    pub(super) fn check_errors_for_stave_id(&mut self, layer_staves_seen: &[LayerStave]) {
        let mut staves_with_errors: Vec<LayerStave> = Vec::new();
        let re: regex::Regex =
            regex::Regex::new("FEE(?:.|)ID:(?P<fee_id>[1-9][0-9]{0,4})").unwrap();

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

    pub(super) fn total_errors(&self) -> u64 {
        self.total_errors
    }

    pub(super) fn add_reported_error(&mut self, error_msg: String) {
        self.total_errors += 1;
        self.reported_errors.push(error_msg);
    }

    pub(super) fn add_custom_check_error(&mut self, error_msg: String) {
        self.total_errors += 1;
        self.custom_checks_stats_errors.push(error_msg);
    }

    pub(super) fn add_fatal_error(&mut self, error_msg: String) {
        self.fatal_error = Some(error_msg);
    }

    pub(super) fn is_fatal_error(&self) -> bool {
        self.fatal_error.is_some()
    }

    pub(super) fn take_fatal_error(&mut self) -> String {
        self.fatal_error.take().expect("No fatal error found!")
    }

    pub(super) fn unique_error_codes_as_slice(&mut self) -> &[u16] {
        if self.unique_error_codes.is_some() {
            return self.unique_error_codes.as_ref().unwrap();
        } else {
            self.process_unique_error_codes();

            self.unique_error_codes
                .as_ref()
                .expect("Unique error codes were never set")
        }
    }

    /// Returns a slice of Layer/Staves seen in error messages
    ///
    /// Returns ´None´ if no staves were seen in any error messages
    /// Panics if staves with errors is None when there's errors.
    pub(super) fn staves_with_errors_as_slice(&self) -> Option<&[LayerStave]> {
        if self.staves_with_errors.is_none() {
            if self.total_errors == 0 {
                return None;
            } else {
                panic!("Staves with errors were never set")
            }
        }
        Some(self.staves_with_errors.as_ref().unwrap())
    }

    pub(super) fn consume_reported_errors(&mut self) -> Vec<String> {
        // Sort stats by memory position where they were found before consuming them
        self.sort_error_msgs_by_mem_pos();
        let mut errors = std::mem::take(&mut self.reported_errors);
        let mut custom_checks_stats_errors = std::mem::take(&mut self.custom_checks_stats_errors);
        errors.append(&mut custom_checks_stats_errors);
        errors
    }
}

fn extract_unique_error_codes(error_messages: &[String]) -> Vec<u16> {
    let mut error_codes: Vec<u16> = Vec::new();
    let re = regex::Regex::new(r"0x.*: \[E(?P<err_code>[0-9]{2,4})\]").unwrap();
    error_messages.iter().for_each(|err_msg| {
        let err_code_match: regex::Captures = re
            .captures(err_msg)
            .unwrap_or_else(|| panic!("Error parsing error code from error msg: {err_msg}"));

        let err_code = err_code_match["err_code"].parse::<u16>().unwrap();
        if !error_codes.contains(&err_code) {
            error_codes.push(err_code);
        }
    });
    error_codes
}
