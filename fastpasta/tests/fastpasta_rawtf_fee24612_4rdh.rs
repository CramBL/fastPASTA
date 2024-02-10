use crate::util::*;
mod util;

// Asserts that the end of processing report summary contains correct information
fn validate_report_summary(byte_output: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let match_patterns = vec![
        "Trigger Type.*0x6A03",
        "RDH.*Version.*7",
        "Total.*RDHs.*4",
        "Total.*hbfs.*2",
        "((layers)|(staves)).*((layers)|(staves)).*L6_36",
    ];
    for pattern in match_patterns {
        match_on_out(false, byte_output, pattern, 1)?;
    }

    Ok(())
}

#[test]
fn check_sanity() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_RAWTF_FEE_24612_4RDHS)
        .arg("check")
        .arg("sanity");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;

    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

#[test]
/// Warns because the detector field of the 2nd RDH changes from 0x0 -> 0xD because of fatal lane errors
fn check_all() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_RAWTF_FEE_24612_4RDHS).arg("check").arg("all");
    cmd.assert().success();

    match_on_out(
        false,
        &cmd.output()?.stderr,
        "WARN Detector field changed",
        1,
    )?;
    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

#[test]
/// Warns because the detector field of the 2nd RDH changes from 0x0 -> 0xD because of fatal lane errors
fn check_all_its() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_RAWTF_FEE_24612_4RDHS)
        .arg("check")
        .arg("all")
        .arg("its")
        .arg("-v4");
    cmd.assert().success();

    match_on_out(
        false,
        &cmd.output()?.stderr,
        "WARN Detector field changed",
        1,
    )?;
    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

#[test]
/// Test that the thread panic from issue 39 is resolved: https://gitlab.cern.ch/mkonig/fastpasta/-/issues/39
fn check_all_its_stave() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_RAWTF_FEE_24612_4RDHS)
        .arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("-v4");
    cmd.assert().success();

    match_on_out(true, &cmd.output()?.stderr, "ERROR Analysis thread", 0)?;
    match_on_out(false, &cmd.output()?.stderr, "thread.*panicked", 0)?;
    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}
