use predicates::str::contains;

use crate::util::*;
mod util;

// Asserts that the end of processing report summary contains correct information
fn validate_report_summary(byte_output: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let match_patterns = vec![
        "Trigger Type.*0x6803",
        "RDH.*Version.*7",
        "Total.*RDHs.*3",
        "Total.*hbfs.*1",
        "((layers)|(staves)).*((layers)|(staves)).*L6_1",
    ];
    for pattern in match_patterns {
        match_on_out_no_case(byte_output, pattern, 1)?;
    }

    Ok(())
}

#[test]
fn check_sanity() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_RAWTF_EPN180_L6_1).arg("check").arg("sanity");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;

    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

#[test]
fn check_all_its_stave_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_RAWTF_EPN180_L6_1)
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

    const EXPECTED_ALPIDE_ERRORS: u8 = 18;

    cmd.arg(FILE_RAWTF_EPN180_L6_1)
        .arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("--filter-its-stave")
        .arg("L6_1");
    cmd.assert().success();

    match_on_out_no_case(
        &cmd.output()?.stdout,
        &format!("errors.*{EXPECTED_ALPIDE_ERRORS}"),
        1,
    )?;

    match_on_out_no_case(
        &cmd.output()?.stderr,
        // Errors that have ALPIDE and lane 66 in them
        "error - 0x.*alpide.*lane.*66",
        EXPECTED_ALPIDE_ERRORS.into(),
    )?;
    match_on_out_no_case(&cmd.output()?.stdout, ".*stave.*l6_1", 1)?;

    Ok(())
}

#[test]
fn check_all_its_stave_trigger_period() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    const EXPECTED_ALPIDE_ERRORS: u8 = 18;

    cmd.arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("--filter-its-stave")
        .arg("L6_1")
        .arg(FILE_RAWTF_EPN180_L6_1)
        .arg("--its-trigger-period")
        .arg("198")
        .arg("-v4");
    cmd.assert().success();

    match_on_out_no_case(
        &cmd.output()?.stderr,
        "error - 0x",
        EXPECTED_ALPIDE_ERRORS.into(),
    )?;
    match_on_out_no_case(&cmd.output()?.stdout, ".*stave.*l6_1", 1)?;

    Ok(())
}

#[test]
fn check_all_its_stave_bad_trigger_period() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    const EXPECTED_ALPIDE_ERRORS: u8 = 18;
    const EXPECTED_TRIGGER_PERIOD_ERRORS: u8 = 18 - 1; // There's 18 TDTs, but a period is from one TDT to the next so there's N-1 periods.

    cmd.arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("--filter-its-stave")
        .arg("L6_1")
        .arg(FILE_RAWTF_EPN180_L6_1)
        .arg("--its-trigger-period")
        .arg("1337")
        .arg("-v4");
    cmd.assert().success();

    match_on_out_no_case(
        &cmd.output()?.stderr,
        "error - 0x",
        (EXPECTED_ALPIDE_ERRORS + EXPECTED_TRIGGER_PERIOD_ERRORS).into(),
    )?;
    match_on_out_no_case(&cmd.output()?.stdout, ".*stave.*l6_1", 1)?;

    Ok(())
}
