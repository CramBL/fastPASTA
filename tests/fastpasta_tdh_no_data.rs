use predicates::str::contains;

use crate::util::*;
mod util;

// Asserts that the end of processing report summary contains correct information
fn validate_report_summary(byte_output: &Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let match_patterns = vec![
        "(?i)Trigger Type.*0x6803",
        "(?i)RDH.*Version.*7",
        "(?i)Total.*RDHs.*3",
        "(?i)Total.*hbfs.*1",
        "(?i)((layers)|(staves)).*((layers)|(staves)).*L0_0",
    ];
    match_patterns.into_iter().for_each(|pattern| {
        assert!(match_on_output(byte_output, pattern, 1));
    });
    Ok(())
}

#[test]
fn check_sanity() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_TDH_NO_DATA).arg("check").arg("sanity");
    cmd.assert().success();

    assert!(match_on_output(&cmd.output()?.stderr, "(?i)error - ", 0));
    assert!(match_on_output(&cmd.output()?.stderr, "(?i)warn - ", 0));

    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

// https://gitlab.cern.ch/mkonig/fastpasta/-/issues/26
#[test]
fn check_sanity_its_issue_26() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_TDH_NO_DATA)
        .arg("check")
        .arg("sanity")
        .arg("its");
    cmd.assert().success();

    assert!(match_on_output(&cmd.output()?.stderr, "(?i)error - ", 0));
    assert!(match_on_output(&cmd.output()?.stderr, "(?i)warn - ", 0));
    validate_report_summary(&cmd.output()?.stdout)?;

    assert!(match_on_output(&cmd.output()?.stdout, "(?i)errors.*0", 1));

    Ok(())
}

// https://gitlab.cern.ch/mkonig/fastpasta/-/issues/26
#[test]
fn check_all_its_issue_26() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_TDH_NO_DATA).arg("check").arg("all").arg("its");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;
    validate_report_summary(&cmd.output()?.stdout)?;

    assert!(match_on_output(&cmd.output()?.stdout, "(?i)errors.*0", 1));

    Ok(())
}

#[test]
fn check_view_its_readout_frames() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_TDH_NO_DATA)
        .arg("view")
        .arg("its-readout-frames");
    cmd.assert().success();

    assert!(match_on_output(&cmd.output()?.stderr, "(?i)error - ", 0));
    assert!(match_on_output(&cmd.output()?.stderr, "(?i)warn - ", 0));

    // Before issue 29 was fixed, 0x5320 was erroneously displayed as a TDH.
    // Here we confirm that it is now correctly interpreted as an IHW
    cmd.assert().stdout(
        contains("RDH").count(3).and(
            contains("TDT").count(36).and(
                contains("TDH")
                    .count(53)
                    .and(contains("IHW").count(2).and(contains("DDW").count(1))),
            ),
        ),
    );

    Ok(())
}

#[test]
fn check_all_its_stave() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_TDH_NO_DATA)
        .arg("check")
        .arg("all")
        .arg("its")
        .arg("--filter-its-stave")
        .arg("l0_0");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;

    match_on_out_no_case(&cmd.output()?.stdout, "errors.*0", 1)?;

    Ok(())
}
