use predicates::str::contains;

use crate::util::*;
mod util;

// Asserts that the end of processing report summary contains correct information
fn validate_report_summary(byte_output: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let match_patterns = vec![
        "Trigger Type.*0x6803",
        "RDH.*Version.*7",
        "Total.*RDHs.*15",
        "Total.*hbfs.*5",
        "((layers)|(staves)).*((layers)|(staves)).*L6_11",
    ];
    for pattern in match_patterns {
        match_on_out_no_case(byte_output, pattern, 1)?;
    }

    Ok(())
}

#[test]
fn check_sanity() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_TDH_NO_DATA_IHW).arg("check").arg("sanity");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;

    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

// https://gitlab.cern.ch/mkonig/fastpasta/-/issues/29
#[test]
fn check_sanity_its_issue_29() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_TDH_NO_DATA_IHW)
        .arg("check")
        .arg("sanity")
        .arg("its");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;

    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

// https://gitlab.cern.ch/mkonig/fastpasta/-/issues/29
#[test]
fn check_all_its_issue_29() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_TDH_NO_DATA_IHW)
        .arg("check")
        .arg("sanity")
        .arg("its");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;

    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

// https://gitlab.cern.ch/mkonig/fastpasta/-/issues/29
#[test]
fn check_view_hbf_issue_29() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_TDH_NO_DATA_IHW).arg("view").arg("hbf");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;

    // Before issue 29 was fixed, 0x5320 was erroneously displayed as a TDH.
    // Here we confirm that it is now correctly interpreted as an IHW
    cmd.assert().stdout(
        contains("5320: IHW")
            .count(1)
            .and(contains("5320: IHW [FF 3F 00 00 00 00 00 00 00 E0]")),
    );

    Ok(())
}

// https://gitlab.cern.ch/mkonig/fastpasta/-/issues/29
#[test]
fn check_view_its_readout_frames_issue_29() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_TDH_NO_DATA_IHW)
        .arg("view")
        .arg("its-readout-frames");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;

    // Before issue 29 was fixed, 0x5320 was erroneously displayed as a TDH.
    // Here we confirm that it is now correctly interpreted as an IHW
    cmd.assert().stdout(
        contains("5320: IHW")
            .count(1)
            .and(contains("5320: IHW [FF 3F 00 00 00 00 00 00 00 E0]")),
    );

    Ok(())
}

#[test]
fn check_all_its_stave_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_TDH_NO_DATA_IHW)
        .arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("--filter-its-stave")
        .arg("l0_0");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;

    match_on_out_no_case(&cmd.output()?.stdout, "errors.*0", 1)?;
    match_on_out_no_case(&cmd.output()?.stdout, ".*not found.*l0_0", 1)?;

    Ok(())
}

#[test]
fn check_all_its_stave() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_TDH_NO_DATA_IHW)
        .arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("--filter-its-stave")
        .arg("L6_11");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;

    match_on_out_no_case(&cmd.output()?.stdout, "errors.*0", 1)?;
    match_on_out_no_case(&cmd.output()?.stdout, ".*stave.*l6_11", 1)?;

    Ok(())
}

#[test]
fn check_all_its_stave_trigger_period_invalid_config() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg("check")
        .arg("all")
        .arg("its")
        .arg("--filter-its-stave")
        .arg("L6_11")
        .arg(FILE_TDH_NO_DATA_IHW)
        .arg("--its-trigger-period")
        .arg("198");
    cmd.assert().failure();

    match_on_out_no_case(&cmd.output()?.stderr, "invalid.*its_stave.*command", 1)?;

    Ok(())
}

#[test]
fn check_all_its_stave_trigger_period() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("--filter-its-stave")
        .arg("L6_11")
        .arg(FILE_TDH_NO_DATA_IHW)
        .arg("--its-trigger-period")
        .arg("198")
        .arg("-v4");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;

    match_on_out_no_case(&cmd.output()?.stdout, "errors.*0", 1)?;
    match_on_out_no_case(&cmd.output()?.stdout, ".*stave.*l6_11", 1)?;

    Ok(())
}
