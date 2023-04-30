use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

/// Path to test files : tests/regression/test-data/
/// Files
pub const FILE_10_RDH: &str = "tests/regression/test-data/10_rdh.raw";
pub const FILE_ERR_NOT_HBF: &str = "tests/regression/test-data/err_not_hbf.raw";
const FILE_THRS_CDW_LINKS: &str = "tests/regression/test-data/thrs_cdw_links.raw";
pub const FILE_READOUT_SUPERPAGE_1: &str = "tests/regression/test-data/readout.superpage.1.raw";
pub const FILE_1_HBF_BAD_CDP_STRUCTURE: &str =
    "tests/regression/test-data/1_hbf_bad_cdp_structure.raw";
const FILE_1_HBF_BAD_DW_DDW0: &str = "tests/regression/test-data/1_hbf_bad_dw_ddw0.raw";
const FILE_1_HBF_BAD_IHW_TDH: &str = "tests/regression/test-data/1_hbf_bad_ihw_tdh.raw";
const FILE_1_HBF_BAD_ITS_PAYLOAD: &str = "tests/regression/test-data/1_hbf_bad_its_payload.raw";
const FILE_1_HBF_BAD_TDT: &str = "tests/regression/test-data/1_hbf_bad_tdt.raw";

/// Helper function to match the raw output of stderr or stdout, with a pattern a fixed amount of times
pub fn match_on_output(byte_output: &Vec<u8>, re_str: &str, match_count: usize) -> bool {
    // Build regex pattern
    let re = fancy_regex::Regex::new(re_str).unwrap();
    // Make the predicate function
    let pred_regex = predicate::function(|&x| re.find_iter(x).count() == match_count);
    // Convert the output to string as utf-8
    let str_res = std::str::from_utf8(&byte_output).expect("invalid utf-8 sequence");
    // Evaluate the output with the predicate
    pred_regex.eval(&str_res)
}

/// Test that all test data files can be parsed successfully
#[test]
fn file_exists_exit_successful_10_rdh() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_10_RDH).arg("check").arg("sanity").arg("-v2");
    cmd.assert().success();

    // Take the output of stderr and match it with a pattern once
    assert!(match_on_output(
        &cmd.output().unwrap().stderr,
        "(?i)exit success",
        1
    ));

    Ok(())
}

#[test]
fn file_exists_exit_successful_err_not_hbf() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_ERR_NOT_HBF)
        .arg("check")
        .arg("sanity")
        .arg("-v2");
    cmd.assert().success();

    assert!(match_on_output(
        &cmd.output().unwrap().stderr,
        "(?i)exit success",
        1
    ));

    Ok(())
}

#[test]
fn file_exists_exit_successful_thrs_cdw_links() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_THRS_CDW_LINKS)
        .arg("check")
        .arg("sanity")
        .arg("-v2");
    cmd.assert().success();

    assert!(match_on_output(
        &cmd.output().unwrap().stderr,
        "(?i)exit success",
        1
    ));

    Ok(())
}

#[test]
fn file_exists_exit_successful_readout_superpage_1() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_READOUT_SUPERPAGE_1)
        .arg("check")
        .arg("sanity")
        .arg("-v2");
    cmd.assert().success();

    assert!(match_on_output(
        &cmd.output().unwrap().stderr,
        "(?i)exit success",
        1
    ));

    Ok(())
}

#[test]
fn file_exists_exit_successful_1_hbf_bad_cdp_structure() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_CDP_STRUCTURE)
        .arg("check")
        .arg("sanity")
        .arg("-v2");
    cmd.assert().success();

    assert!(match_on_output(
        &cmd.output().unwrap().stderr,
        "(?i)exit success",
        1
    ));

    Ok(())
}

#[test]
fn file_exists_exit_successful_1_hbf_bad_dw_ddw0() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_DW_DDW0)
        .arg("check")
        .arg("sanity")
        .arg("-v2");
    cmd.assert().success();

    assert!(match_on_output(
        &cmd.output().unwrap().stderr,
        "(?i)exit success",
        1
    ));

    Ok(())
}

#[test]
fn file_exists_exit_successful_1_hbf_bad_ihw_tdh() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_IHW_TDH)
        .arg("check")
        .arg("sanity")
        .arg("-v2");
    cmd.assert().success();

    assert!(match_on_output(
        &cmd.output().unwrap().stderr,
        "(?i)exit success",
        1
    ));

    Ok(())
}

#[test]
fn file_exists_exit_successful_1_hbf_bad_its_payload() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_ITS_PAYLOAD)
        .arg("check")
        .arg("sanity")
        .arg("-v2");
    cmd.assert().success();

    assert!(match_on_output(
        &cmd.output().unwrap().stderr,
        "(?i)exit success",
        1
    ));

    Ok(())
}

#[test]
fn file_exists_exit_successful_1_hbf_bad_tdt() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_TDT)
        .arg("check")
        .arg("sanity")
        .arg("-v2");
    cmd.assert().success();

    assert!(match_on_output(
        &cmd.output().unwrap().stderr,
        "(?i)exit success",
        1
    ));

    Ok(())
}

/// Check that a not found file returns a fatal error, with a description of an OS error
///
/// Try with all the different verbosity values 0-4
#[test]
fn file_doesnt_exist_v0() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg("test/file/doesnt/exist")
        .arg("check")
        .arg("sanity")
        .arg("-v0");
    cmd.assert().failure().stderr(
        predicate::str::contains("ERROR - FATAL:").and(predicate::str::contains("os error")),
    );
    Ok(())
}
#[test]
fn file_doesnt_exist_v1() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg("test/file/doesnt/exist")
        .arg("check")
        .arg("sanity")
        .arg("-v1");

    cmd.assert().failure().stderr(
        predicate::str::contains("ERROR - FATAL:").and(predicate::str::contains("os error")),
    );

    Ok(())
}

#[test]
fn file_doesnt_exist_v2() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg("test/file/doesnt/exist")
        .arg("check")
        .arg("sanity")
        .arg("-v2");

    cmd.assert().failure().stderr(
        predicate::str::contains("ERROR - FATAL:").and(predicate::str::contains("os error")),
    );

    Ok(())
}

#[test]
fn file_doesnt_exist_v3() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg("test/file/doesnt/exist")
        .arg("check")
        .arg("sanity")
        .arg("-v3");

    cmd.assert().failure().stderr(
        predicate::str::contains("ERROR - FATAL:").and(predicate::str::contains("os error")),
    );
    Ok(())
}

#[test]
fn file_doesnt_exist_v4() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg("test/file/doesnt/exist")
        .arg("check")
        .arg("sanity")
        .arg("-v4");

    cmd.assert().failure().stderr(
        predicate::str::contains("ERROR - FATAL:").and(predicate::str::contains("os error")),
    );

    Ok(())
}
