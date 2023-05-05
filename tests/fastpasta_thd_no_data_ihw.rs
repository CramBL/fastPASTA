use predicates::str::contains;

use crate::util::*;
mod util;

// Asserts that the end of processing report summary contains correct information
fn validate_report_summary(byte_output: &Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let match_patterns = vec![
        "(?i)Trigger Type.*0x6803",
        "(?i)RDH.*Version.*7",
        "(?i)Total.*RDHs.*15",
        "(?i)Total.*hbfs.*5",
        "(?i)((layers)|(staves)).*((layers)|(staves)).*L6_11",
    ];
    match_patterns.into_iter().for_each(|pattern| {
        assert!(match_on_output(byte_output, pattern, 1));
    });
    Ok(())
}

#[test]
fn check_sanity() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_TDH_NO_DATA_IHW).arg("check").arg("sanity");
    cmd.assert().success();

    assert!(match_on_output(&cmd.output()?.stderr, "(?i)error - ", 0));
    assert!(match_on_output(&cmd.output()?.stderr, "(?i)warn - ", 0));

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

    assert!(match_on_output(&cmd.output()?.stderr, "(?i)error - ", 0));
    assert!(match_on_output(&cmd.output()?.stderr, "(?i)warn - ", 0));

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

    assert!(match_on_output(&cmd.output()?.stderr, "(?i)error - ", 0));
    assert!(match_on_output(&cmd.output()?.stderr, "(?i)warn - ", 0));

    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

// https://gitlab.cern.ch/mkonig/fastpasta/-/issues/29
#[test]
fn check_view_hbf_issue_29() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_TDH_NO_DATA_IHW).arg("view").arg("hbf");
    cmd.assert().success();

    assert!(match_on_output(&cmd.output()?.stderr, "(?i)error - ", 0));
    assert!(match_on_output(&cmd.output()?.stderr, "(?i)warn - ", 0));

    // Before issue 29 was fixed, 0x5320 was erroneously displayed as a TDH.
    // Here we confirm that it is now correctly interpreted as an IHW
    cmd.assert().stdout(
        contains("5320: IHW")
            .count(1)
            .and(contains("5320: IHW [FF 3F 00 00 00 00 00 00 00 E0]")),
    );

    Ok(())
}
