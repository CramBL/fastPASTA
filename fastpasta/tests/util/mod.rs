#![allow(dead_code)]
/// Re-export some common utilities for system tests
pub use assert_cmd::prelude::*; // Add methods on commands
pub use assert_cmd::Command; // Get the methods for the Commands struct
pub use assert_fs::prelude::*;
pub use assert_fs::TempDir;
pub use predicate::str::is_match;
pub use predicates::prelude::*; // Used for writing assertions // Create temporary directories

/// Path to test files : tests/test-data/
/// Files
pub const FILE_10_RDH: &str = "../tests/test-data/10_rdh.raw";
pub const FILE_ERR_NOT_HBF: &str = "../tests/test-data/err_not_hbf.raw";
pub const FILE_THRS_CDW_LINKS: &str = "../tests/test-data/thrs_cdw_links.raw";
pub const FILE_READOUT_SUPERPAGE_1: &str = "../tests/test-data/readout.superpage.1.raw";
pub const FILE_1_HBF_BAD_CDP_STRUCTURE: &str = "../tests/test-data/1_hbf_bad_cdp_structure.raw";
pub const FILE_1_HBF_BAD_DW_DDW0: &str = "../tests/test-data/1_hbf_bad_dw_ddw0.raw";
pub const FILE_1_HBF_BAD_IHW_TDH: &str = "../tests/test-data/1_hbf_bad_ihw_tdh.raw";
pub const FILE_1_HBF_BAD_ITS_PAYLOAD: &str = "../tests/test-data/1_hbf_bad_its_payload.raw";
pub const FILE_1_HBF_BAD_TDT: &str = "../tests/test-data/1_hbf_bad_tdt.raw";
pub const FILE_TDH_NO_DATA: &str = "../tests/test-data/tdh_no_data.raw";
pub const FILE_TDH_NO_DATA_IHW: &str = "../tests/test-data/tdh_no_data_ihw.raw";
pub const FILE_RAWTF_EPN180_L6_1: &str = "../tests/test-data/rawtf_epn180_l6_1.raw";

pub const FILE_OUTPUT_TMP: &str = "../tests/test-data/output.tmp";

/// Helper function to match the raw output of stderr or stdout, with a pattern a fixed amount of times
pub fn match_on_output(
    byte_output: &[u8],
    re_str: &str,
    match_count: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    // Build regex pattern
    let re = fancy_regex::Regex::new(re_str).unwrap();
    // Make the predicate function
    let pred_regex = predicate::function(|&x| re.find_iter(x).count() == match_count);
    // Convert the output to string as utf-8
    let str_res = std::str::from_utf8(byte_output).expect("invalid utf-8 sequence");
    // Evaluate the output with the predicate
    assert!(pred_regex.eval(&str_res));
    Ok(())
}

/// Helper function to match the raw output of stderr or stdout, with a pattern a fixed amount of times, case insensitive
pub fn match_on_out_no_case(
    byte_output: &[u8],
    re_str: &str,
    expect_match: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    // Convert the output to string as utf-8
    let str_res = std::str::from_utf8(byte_output).expect("invalid utf-8 sequence");
    // Build regex pattern
    let re = fancy_regex::Regex::new(&("(?i)".to_owned() + re_str)).unwrap();
    // Count the number of matches
    let match_count = re.find_iter(str_res).count();
    // Assert that the number of matches is equal to the expected number of matches
    assert_eq!(
        match_count, expect_match,
        "regex: {re_str} - expected match count: {expect_match}, got {match_count}\nFailed to match on:\n{str_res}"
    );
    Ok(())
}

/// Helper function takes in the output of stderr and asserts that there are no errors or warnings
pub fn assert_no_errors_or_warn(
    stderr_byte_output: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    match_on_out_no_case(stderr_byte_output, "error - ", 0)?;
    match_on_out_no_case(stderr_byte_output, "warn - ", 0)?;
    Ok(())
}

/// Create a custom checks TOML file from with `toml_content` at the specified `toml_path` path
pub fn create_custom_checks_toml(
    toml_content: &str,
    toml_path: &std::path::PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut custom_checks_file = std::fs::File::create(toml_path)?;
    std::io::Write::write_all(&mut custom_checks_file, toml_content.as_bytes())?;
    custom_checks_file.sync_all()?;

    Ok(())
}