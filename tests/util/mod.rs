#![allow(dead_code)]
/// Re-export some common utilities for system tests
pub use assert_cmd::prelude::*; // Add methods on commands
pub use assert_fs::prelude::*;
pub use predicate::str::is_match;
pub use predicates::prelude::*; // Used for writing assertions
pub use std::process::Command; // Run programs

/// Path to test files : tests/test-data/
/// Files
pub const FILE_10_RDH: &str = "tests/test-data/10_rdh.raw";
pub const FILE_ERR_NOT_HBF: &str = "tests/test-data/err_not_hbf.raw";
pub const FILE_THRS_CDW_LINKS: &str = "tests/test-data/thrs_cdw_links.raw";
pub const FILE_READOUT_SUPERPAGE_1: &str = "tests/test-data/readout.superpage.1.raw";
pub const FILE_1_HBF_BAD_CDP_STRUCTURE: &str = "tests/test-data/1_hbf_bad_cdp_structure.raw";
pub const FILE_1_HBF_BAD_DW_DDW0: &str = "tests/test-data/1_hbf_bad_dw_ddw0.raw";
pub const FILE_1_HBF_BAD_IHW_TDH: &str = "tests/test-data/1_hbf_bad_ihw_tdh.raw";
pub const FILE_1_HBF_BAD_ITS_PAYLOAD: &str = "tests/test-data/1_hbf_bad_its_payload.raw";
pub const FILE_1_HBF_BAD_TDT: &str = "tests/test-data/1_hbf_bad_tdt.raw";
pub const FILE_TDH_NO_DATA: &str = "tests/test-data/tdh_no_data.raw";
pub const FILE_TDH_NO_DATA_IHW: &str = "tests/test-data/tdh_no_data_ihw.raw";

pub const FILE_OUTPUT_TMP: &str = "tests/test-data/output.tmp";

/// Helper function to match the raw output of stderr or stdout, with a pattern a fixed amount of times
pub fn match_on_output(
    byte_output: &Vec<u8>,
    re_str: &str,
    match_count: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    // Build regex pattern
    let re = fancy_regex::Regex::new(re_str).unwrap();
    // Make the predicate function
    let pred_regex = predicate::function(|&x| re.find_iter(x).count() == match_count);
    // Convert the output to string as utf-8
    let str_res = std::str::from_utf8(&byte_output).expect("invalid utf-8 sequence");
    // Evaluate the output with the predicate
    assert!(pred_regex.eval(&str_res));
    Ok(())
}

/// Helper function to match the raw output of stderr or stdout, with a pattern a fixed amount of times, case insensitive
pub fn match_on_out_no_case(
    byte_output: &Vec<u8>,
    re_str: &str,
    match_count: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    // Build regex pattern
    let re = fancy_regex::Regex::new(&("(?i)".to_owned() + re_str)).unwrap();
    // Make the predicate function
    let pred_regex = predicate::function(|&x| re.find_iter(x).count() == match_count);
    // Convert the output to string as utf-8
    let str_res = std::str::from_utf8(&byte_output).expect("invalid utf-8 sequence");
    // Evaluate the output with the predicate
    assert!(
        pred_regex.eval(&str_res),
        "regex: {} - expected match count: {match_count}",
        re_str
    );
    Ok(())
}

/// Helper function takes in the output of stderr and asserts that there are no errors or warnings
pub fn assert_no_errors_or_warn(
    stderr_byte_output: &Vec<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
    match_on_out_no_case(stderr_byte_output, "error - ", 0)?;
    match_on_out_no_case(stderr_byte_output, "warn - ", 0)?;
    Ok(())
}
