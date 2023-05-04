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
