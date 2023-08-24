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
        match_on_out_no_case(byte_output, pattern, 1)?;
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
/// Errors because the detector field of the 2nd RDH changes from 0x0 -> 0xD because of fatal lane errors
fn check_all() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_RAWTF_FEE_24612_4RDHS).arg("check").arg("all");
    cmd.assert().success();

    match_on_out_no_case(&cmd.output()?.stderr, "ERROR - 0x360", 1)?;
    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}
