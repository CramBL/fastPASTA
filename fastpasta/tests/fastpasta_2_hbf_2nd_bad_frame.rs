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
        match_on_out(false, byte_output, pattern, 1)?;
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

    match_on_out(
        false,
        &cmd.output()?.stderr,
        prefix_and_then(ERROR_PREFIX, "0x.*[E701]"),
        expect_error_count,
    )?;
    match_on_out(false, &cmd.output()?.stdout, "total errors.*198.*E701", 1)?;
    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

#[test]
/// Test that no error messages is displayed, when filtering by error code 70 and the reported error is code 701
fn check_all_its_stave_error_messages_filtered_out_by_error_code_filter(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    // Expect to see no error messages because we filter by error code 70 and the reported error is code 701
    let expect_error_count = 0;

    cmd.arg(FILE_2_HBF_2ND_BAD_FRAME)
        .arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("-v4")
        .arg("-w")
        .arg("70");
    cmd.assert().success();

    match_on_out(
        false,
        &cmd.output()?.stderr,
        r"ERROR 0x.*[E701]",
        expect_error_count,
    )?;
    match_on_out(false, &cmd.output()?.stdout, "total errors.*198.*E701", 1)?;
    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

#[test]
/// Test that 100 error messages is displayed, when filtering by error code 701 and allowing maximum 100 error messages
fn check_all_its_stave_100_error_message_matching_err_code_filter_and_maximum_allowed(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    // Expect to see 100 error messages because we filter by error code 701 which matches the reported error
    // And we allow maximum 100 error messages to be displayed
    let expect_error_count = 100;
    let err_code_filter = "701";
    let max_err_msg = "100";

    cmd.arg(FILE_2_HBF_2ND_BAD_FRAME)
        .arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("-v4")
        .arg("-w")
        .arg(err_code_filter)
        .arg("-e")
        .arg(max_err_msg);
    cmd.assert().success();

    match_on_out(
        false,
        &cmd.output()?.stderr,
        prefix_and_then(ERROR_PREFIX, "0x.*[E701]"),
        expect_error_count,
    )?;
    match_on_out(false, &cmd.output()?.stdout, "total errors.*198.*E701", 1)?;
    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}
