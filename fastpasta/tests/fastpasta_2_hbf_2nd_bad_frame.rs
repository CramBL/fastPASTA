use crate::util::*;
mod util;

// Asserts that the end of processing report summary contains correct information
fn validate_report_summary(byte_output: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let match_patterns = vec![
        "Trigger Type.*0x6003",
        "RDH.*Version.*7",
        "Total.*RDHs.*6",
        "Total.*hbfs.*2",
        "((layers)|(staves)).*((layers)|(staves)).*L1_8",
    ];
    for pattern in match_patterns {
        match_on_out_no_case(byte_output, pattern, 1)?;
    }

    Ok(())
}

#[test]
fn check_sanity() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_2_HBF_2ND_BAD_FRAME).arg("check").arg("sanity");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;
    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

#[test]
fn check_all() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_2_HBF_2ND_BAD_FRAME).arg("check").arg("all");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;
    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

#[test]
fn check_all_its() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_2_HBF_2ND_BAD_FRAME)
        .arg("check")
        .arg("all")
        .arg("its")
        .arg("-v4");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;
    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

#[test]
fn check_all_its_stave() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    let expect_error_count = 198;

    cmd.arg(FILE_2_HBF_2ND_BAD_FRAME)
        .arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("-v4");
    cmd.assert().success();

    match_on_out_no_case(
        &cmd.output()?.stderr,
        r"ERROR - 0x.*[E701]",
        expect_error_count,
    )?;
    match_on_out_no_case(&cmd.output()?.stdout, "total errors.*198.*E701", 1)?;
    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}
