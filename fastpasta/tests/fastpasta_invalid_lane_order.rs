use crate::util::*;
mod util;

// Asserts that the end of processing report summary contains correct information
fn validate_report_summary(byte_output: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let match_patterns = vec![
        "Trigger Type.*0x6A03",
        "RDH.*Version.*7",
        "Total.*RDHs.*2",
        "Total.*hbfs.*1",
        "((layers)|(staves)).*((layers)|(staves)).*L0_12",
    ];
    for pattern in match_patterns {
        match_on_out(false, byte_output, pattern, 1)?;
    }

    Ok(())
}

#[test]
fn check_sanity() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_INVALID_LANE_ORDER_1HBF)
        .arg("check")
        .arg("sanity");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;

    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

#[test]
fn view_its_readout_frames() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg("view")
        .arg("its-readout-frames")
        .arg(FILE_INVALID_LANE_ORDER_1HBF);

    use predicate::str::contains;
    cmd.assert().success().stdout(
        contains("RDH").count(2).and(
            contains("IHW").count(1).and(
                contains("TDH")
                    .count(1)
                    .and(contains("TDT").count(1).and(contains("DDW").count(1))),
            ),
        ),
    );

    Ok(())
}

#[test]
fn check_all_its_stave() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    const EXPECTED_ERRORS: u8 = 3;

    cmd.arg(FILE_INVALID_LANE_ORDER_1HBF)
        .arg("check")
        .arg("all")
        .arg("its-stave");
    cmd.assert().success();

    match_on_out(
        false,
        &cmd.output()?.stdout,
        &format!("errors.*{EXPECTED_ERRORS}"),
        1,
    )?;

    match_on_out(
        false,
        &cmd.output()?.stderr,
        prefix_and_then(ERROR_PREFIX, "0x.*lane.*5"),
        EXPECTED_ERRORS.into(),
    )?;
    match_on_out(false, &cmd.output()?.stderr, "chip id order", 1)?;

    assert_alpide_stats_report(&cmd.output()?.stdout, 3, 0, 0, 0, 0, 0, 0)?;

    Ok(())
}
