//! Contains the [ErrPrinter] that prints error messages in accordance to a given configuration

use crate::util::*;

/// Prints error messages in accordance to a given configuration
#[derive(Debug, Default)]
pub struct ErrPrinter<'a> {
    max_errors: Option<u32>,
    error_code_filter: Option<&'a [String]>,
}

impl<'a> ErrPrinter<'a> {
    /// Create a new [ErrPrinter] with the given configuration
    pub fn new(max_errors: Option<u32>, error_code_filter: Option<&'a [String]>) -> Self {
        Self {
            max_errors,
            error_code_filter,
        }
    }

    /// Print the error messages in accordance to the configuration
    ///
    /// If an error code filter is supplied, only errors matching the filter are displayed
    /// If the max errors is set, only the first `max_errors` are displayed
    ///
    /// The unique error codes are used to minify the error code filter to avoid doing unnecessary comparisons
    pub fn print<E: Iterator<Item = &'a Box<str>>>(
        &self,
        err_msgs: E,
        unique_error_codes: &[String],
    ) {
        match self.error_code_filter {
            Some(filter) => {
                // Reduce the error code filter to codes that were actually seen
                let min_filter = self.minify_filter(filter, unique_error_codes);
                // Filter the error messages and take the first `max_errors` if set
                self.filter_error_msgs(self.max_errors, &min_filter, err_msgs)
                    .for_each(|e| crate::display_error(e));
            }
            None => {
                // Take the first `max_errors` if set
                err_msgs
                    .take(self.max_errors.unwrap_or(u32::MAX) as usize)
                    .for_each(|e| crate::display_error(e));
            }
        };
    }

    // Reduce the error code filter to codes that were actually seen in the error messages
    fn minify_filter(
        &self,
        error_code_filter: &[String],
        unique_error_codes: &[String],
    ) -> Vec<String> {
        error_code_filter
            .iter()
            .filter(|ec| unique_error_codes.contains(ec))
            .map_into()
            .collect::<Vec<String>>()
    }

    fn filter_error_msgs<'b, E: Iterator<Item = &'a Box<str>> + 'b>(
        &self,
        max_errors: Option<u32>,
        ec_filter: &'b [String],
        err_msgs: E,
    ) -> impl Iterator<Item = &'a Box<str>> + 'b {
        err_msgs
            .filter(move |err_msg| {
                for ec in ec_filter {
                    let mut msg_chars = err_msg.chars();
                    // Advances the iterator until the '[' character and gets the position
                    // This is the position where we start comparing the error code digits
                    if msg_chars
                        .position(|c| c == '[')
                        .map_or(false, |pos_err_code| {
                            match_error_code(err_msg, ec.chars(), msg_chars, pos_err_code)
                        })
                    {
                        return true;
                    }
                }
                false
            })
            .take(max_errors.unwrap_or(u32::MAX) as usize)
    }
}

/// Check if the error code matches the filter (with many opportunities for short circuiting)
/// # Arguments
/// * `err_msg`: the error message
/// * `filter_chars`: the characters of the filter
/// * `err_msg_chars`: the characters of the error message
/// * `pos_err_code`: the position of the '[' character
fn match_error_code(
    err_msg: &str,
    filter_chars: Chars,
    err_msg_chars: Chars,
    pos_err_code: usize,
) -> bool {
    // The position in the err_msg where we are comparing the characters
    let mut pos_char_cmp: usize = pos_err_code;

    // Skip the 'E' in the msg_chars iterator that comes after the '[' character so now we are comparing the error code digits
    debug_assert_eq!(&err_msg_chars.as_str()[0..=0], "E");
    let msg_chars = err_msg_chars.skip(1);
    pos_char_cmp += 1;
    // Compare the error code digits in the filter and the message
    filter_chars
        .zip(msg_chars)
        .all(|(fchar, mchar)| {
            // Increment the position in the err_msg where we are comparing the characters
            pos_char_cmp += 1;
            fchar == mchar
        })
        .then(|| {
            // Check that the next character in the err_msg is a ']'
            pos_char_cmp += 1;
            err_msg
                .chars()
                .nth(pos_char_cmp)
                .map_or(false, |c| c == ']')
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_minify_filter() {
        let err_code_filter = vec!["1".into(), "2".into(), "3".into(), "4".into(), "5".into()];
        let err_printer = ErrPrinter::new(None, Some(&err_code_filter));

        let unique_error_codes: Vec<String> = vec!["1".into(), "5".into(), "6".into(), "7".into()];

        let minified_filter = err_printer.minify_filter(&err_code_filter, &unique_error_codes);

        assert_eq!(minified_filter, vec!["1", "5"]);
    }

    #[test]
    /// Test that the filter is minified correctly when several of the unique error codes are substrings of the filter
    fn test_minify_filter_narrow_unique_errors() {
        let err_code_filter = vec![
            "1".into(),
            "200".into(),
            "3".into(),
            "004".into(),
            "51".into(),
        ];
        let err_printer = ErrPrinter::new(None, Some(&err_code_filter));

        let unique_error_codes: Vec<String> = vec![
            "01".into(),
            "20".into(),
            "6".into(),
            "10".into(),
            "51".into(),
            "04".into(),
            "30".into(),
        ];

        let minified_filter = err_printer.minify_filter(&err_code_filter, &unique_error_codes);

        assert_eq!(minified_filter, vec!["51"]);
    }

    #[test]
    /// Test that the filter is minified correctly when several of the filter codes are substring of the unique error codes
    fn test_minify_filter_wide_unique_error_codes() {
        let err_code_filter = vec![
            "01".into(),
            "20".into(),
            "6".into(),
            "10".into(),
            "51".into(),
            "04".into(),
            "30".into(),
        ];
        let err_printer = ErrPrinter::new(None, Some(&err_code_filter));

        let unique_error_codes: Vec<String> = vec![
            "1".into(),
            "200".into(),
            "3".into(),
            "004".into(),
            "51".into(),
            "100".into(),
        ];

        let minified_filter = err_printer.minify_filter(&err_code_filter, &unique_error_codes);

        assert_eq!(minified_filter, vec!["51"]);
    }

    #[test]
    fn test_filter_error_messages_simple() {
        let err_code_filter = vec!["1".into()];
        let err_printer = ErrPrinter::new(None, Some(&err_code_filter));

        let err_msgs = vec![
            "Error message [E1] 1st of should be filtered".into(),
            "Error message [E2]".into(),
            "Error message [E1] 2nd of should be filtered".into(),
            "Error message [E4]".into(),
        ];

        let filtered_err_msgs: Vec<&Box<str>> = err_printer
            .filter_error_msgs(None, &err_code_filter, err_msgs.iter())
            .collect();

        assert_eq!(
            filtered_err_msgs,
            vec![
                &"Error message [E1] 1st of should be filtered".into(),
                &"Error message [E1] 2nd of should be filtered".into(),
            ]
        );
    }

    #[test]
    fn test_filter_error_messages() {
        let err_code_filter = vec!["1".into(), "2".into(), "3".into(), "4".into(), "5".into()];
        let err_printer = ErrPrinter::new(None, Some(&err_code_filter));

        let err_msgs = vec![
            "Error message [E1]".into(),
            "Error message [E2]".into(),
            "Error message [E3]".into(),
            "Error message [E4]".into(),
            "Error message [E5]".into(),
            "Error message [E6]".into(),
            "Error message [E2]".into(),
            "Error message [E7]".into(),
            "Error message [E8]".into(),
            "Error message [E9]".into(),
            "Error message [E01]".into(),
            "Error message [E100]".into(),
        ];

        let filtered_err_msgs: Vec<&Box<str>> = err_printer
            .filter_error_msgs(None, &err_code_filter, err_msgs.iter())
            .collect();

        assert_eq!(
            filtered_err_msgs,
            vec![
                &"Error message [E1]".into(),
                &"Error message [E2]".into(),
                &"Error message [E3]".into(),
                &"Error message [E4]".into(),
                &"Error message [E5]".into(),
                &"Error message [E2]".into(),
            ]
        );
    }
}
