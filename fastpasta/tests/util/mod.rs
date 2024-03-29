#![allow(dead_code)]
use std::fmt::Display;

/// Re-export some common utilities for system tests
pub use assert_cmd::prelude::*; // Add methods on commands
pub use assert_cmd::Command;
pub use assert_fs::fixture::ChildPath;
// Get the methods for the Commands struct
pub use assert_fs::prelude::*;
pub use assert_fs::TempDir;
#[allow(unused_imports)]
pub use predicates::prelude::*; // Used for writing assertions // Create temporary directories
#[allow(unused_imports)]
use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};
use regex::RegexBuilder;

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
pub const FILE_RAWTF_FEE_24612_4RDHS: &str = "../tests/test-data/o2_rawtf_fee24612_4rdh.raw";
pub const FILE_INVALID_LANE_ORDER_1HBF: &str = "../tests/test-data/invalid_lane_order_1hbf.raw";
pub const FILE_CI_OLS_DATA_1HBF: &str = "../tests/test-data/ci_ols_data_1hbf.raw"; // has chip ordering problem because one chip is disabled
pub const FILE_2_RDH_DET_FIELD_V1_21_0: &str = "../tests/test-data/2_rdh_det_field_v1.21.0.raw"; // has Detector field v1.21.0
pub const FILE_2_HBF_2ND_BAD_FRAME: &str = "../tests/test-data/2_hbf_2nd_bad_frame.raw"; // First HBF is valid but second lacks data words even though no error has been indicated with APE/TDT/DDW
pub const FILE_12_LINKS_2HBF: &str = "../tests/test-data/12_links_2hbf.raw"; // 12 links with 1 HBF each

/// matches a single ANSI escape code
pub const ANSI_ESCAPE_REGEX: &str = r"(\x9B|\x1B\[)[0-?]*[ -\/]*[@-~]";
/// WARN prefix with an ANSI escape code
pub const WARN_PREFIX: &str = concat!("WARN ", r"(\x9B|\x1B\[)[0-?]*[ -\/]*[@-~]");
pub const ERROR_PREFIX: &str = concat!("ERROR ", r"(\x9B|\x1B\[)[0-?]*[ -\/]*[@-~]");

/// Helper function to create a regex pattern with a prefix and a suffix
pub fn prefix_and_then<S>(prefix: &'static str, suffix: S) -> String
where
    S: AsRef<str> + Display,
{
    format!("{prefix}{suffix}")
}

/// Regex pattern that should match as many times as there are RDHs in the file
/// Matches the RDH version (7 or 6), the header size (64), and the data format (0 or 2).
pub const VIEW_RDH_REGEX_SANITY: &str = ":.*(7|6).*64.*(0|2)";

/// Helper function to create a temp dir and a child file path
///
/// It's important to return the directory because the directory must be kept alive (don't match dir with '_' it will drop it immediately)
pub fn make_tmp_dir_w_fpath() -> (TempDir, ChildPath) {
    let tmp_d = TempDir::new().unwrap();
    let tmp_fpath = tmp_d.child("tmp.raw");
    (tmp_d, tmp_fpath)
}

/// Helper function to create a temp dir and a child file path with a given name
///
/// It's important to return the directory because the directory must be kept alive (don't match dir with '_' it will drop it immediately)
pub fn make_tmp_dir_w_named_file(fname: &str) -> (TempDir, ChildPath) {
    let tmp_d = TempDir::new().unwrap();
    let tmp_fpath = tmp_d.child(fname);
    (tmp_d, tmp_fpath)
}

/// Helper function to read output stats from a file
///
/// Takes a reference to a `ChildPath` to enforce temp file usage
pub fn read_stats_from_file(
    stats_fpath: &ChildPath,
    format: &str,
) -> Result<fastpasta::stats::stats_collector::StatsCollector, Box<dyn std::error::Error>> {
    let stats_file = std::fs::read_to_string(stats_fpath)?;
    let stats = match format.to_uppercase().as_str() {
        "JSON" => serde_json::from_str(&stats_file)?,
        "TOML" => toml::from_str(&stats_file)?,
        _ => panic!("Invalid format: {format}"),
    };

    Ok(stats)
}

/// Helper function to match the raw output of stderr or stdout, with a pattern a fixed amount of times, case insensitive
pub fn match_on_out<S>(
    case_sensitive: bool,
    byte_output: &[u8],
    re: S,
    expect_match: usize,
) -> Result<(), Box<dyn std::error::Error>>
where
    S: AsRef<str> + ToOwned + Display + Into<String>,
{
    // Convert the output to string as utf-8
    let str_res = std::str::from_utf8(byte_output).expect("invalid utf-8 sequence");
    // Build regex pattern
    let regex_pattern = if case_sensitive {
        re.to_string()
    } else {
        format!("(?i){re}")
    };
    let re = fancy_regex::Regex::new(&regex_pattern).unwrap();
    // Count the number of matches
    let match_count = re.find_iter(str_res).count();
    // Assert that the number of matches is equal to the expected number of matches
    assert_eq!(
        match_count, expect_match,
        "regex: {re} - expected match count: {expect_match}, got {match_count}\nFailed to match on:\n{str_res}"
    );
    Ok(())
}

/// Helper function takes in the output of stderr and asserts that there are no errors, warnings, or thread panics.
pub fn assert_no_errors_or_warn(
    stderr_byte_output: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    match_on_out(true, stderr_byte_output, ERROR_PREFIX, 0)?;
    match_on_out(true, stderr_byte_output, WARN_PREFIX, 0)?;
    match_on_out(false, stderr_byte_output, "thread.*panicked", 0)?;
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

/// Helper to build a case-insensitive regex pattern and assert that there's a match
/// Used to check for some pattern ending in some value (e.g. "chip.*trailers.*seen.*{expect_cnt}")
/// Takes a string slice as input to prevent doing the `from_utf8` conversion multiple times
fn match_count_suffix(
    haystack: &str,
    pre_re: &str,
    expect_cnt: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    // Build regex pattern
    let re = RegexBuilder::new(&format!("{pre_re}{expect_cnt}"))
        .case_insensitive(true)
        .build()
        .unwrap();
    assert!(
        re.is_match(haystack),
        "regex: {re}\nFailed to match on:\n{haystack}"
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
/// Assert that the output of the alpide stats report matches the expected values
pub fn assert_alpide_stats_report(
    byte_output: &[u8],
    chip_trailers_seen: u64,
    busy_violations: u64,
    data_overrun: u64,
    transmission_in_fatal: u64,
    flushed_incomplete: u64,
    strobe_extended: u64,
    busy_transitions: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    // Convert the output to string as utf-8
    let str_res = std::str::from_utf8(byte_output).expect("invalid utf-8 sequence");

    // Check that the output contains the expected values (short circuits on first failure)
    match_count_suffix(str_res, "chip.*trailers.*seen.*", chip_trailers_seen)?;
    match_count_suffix(str_res, "busy.*violations.*", busy_violations)?;
    match_count_suffix(str_res, "data.*overrun.*", data_overrun)?;
    match_count_suffix(str_res, "transmission.*in.*fatal.*", transmission_in_fatal)?;
    match_count_suffix(str_res, "flushed.*incomplete.*", flushed_incomplete)?;
    match_count_suffix(str_res, "strobe.*extended.*", strobe_extended)?;
    match_count_suffix(str_res, "busy.*transitions.*", busy_transitions)?;
    Ok(())
}
