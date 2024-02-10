use predicates::str::is_match;

use crate::util::*;
mod util;

const MATCH_PATTERNS: [&str; 6] = [
    "Trigger Type.*0x6A03",
    "Trigger Type.*SOC",
    "RDH.*Version.*7",
    "Total.*RDHs.*2",
    "Total.*hbfs.*0",
    "((layers)|(staves)).*((layers)|(staves)).*L0_12",
];

#[test]
fn err_not_hbf_detect_page_error() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_ERR_NOT_HBF).arg("check").arg("all");
    cmd.assert().success();

    match_on_out(
        false,
        &cmd.output()?.stderr,
        prefix_and_then(ERROR_PREFIX, "0xa0.*pages"),
        1,
    )?;

    Ok(())
}

#[test]
fn view_rdh() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_ERR_NOT_HBF).arg("view").arg("rdh");

    cmd.assert()
        .success()
        .stdout(is_match(VIEW_RDH_REGEX_SANITY)?.count(2));

    Ok(())
}

#[test]
fn check_sanity() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_ERR_NOT_HBF).arg("check").arg("sanity");
    cmd.assert().success();

    for pattern in MATCH_PATTERNS {
        match_on_out(false, &cmd.output()?.stdout, pattern, 1)?;
    }

    Ok(())
}

#[test]
fn check_sanity_debug_verbosity() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_ERR_NOT_HBF)
        .arg("check")
        .arg("sanity")
        .arg("-v3");

    cmd.assert().success();

    match_on_out(false, &cmd.output()?.stderr, "run.*type.*SOC", 1)?;

    Ok(())
}

#[test]
fn view_its_readout_frames() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_ERR_NOT_HBF)
        .arg("view")
        .arg("its-readout-frames");

    use predicate::str::contains;
    cmd.assert().success().stdout(
        contains("RDH").count(2).and(
            contains("IHW").count(2).and(
                contains("TDH").count(2).and(
                    contains("TDT")
                        .count(2)
                        .and(contains("DDW").count(0).and(contains("stop=0").count(2))),
                ),
            ),
        ),
    );

    Ok(())
}

#[test]
fn check_all_its_stave_debug_verbosity() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_ERR_NOT_HBF)
        .arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("-v3");

    cmd.assert().success();

    match_on_out(false, &cmd.output()?.stderr, "run.*type.*SOC", 1)?;
    // Check the total errors statistic in the report contains the E11 error code.
    match_on_out(false, &cmd.output()?.stdout, "total.*errors.*E11", 1)?;

    assert_alpide_stats_report(&cmd.output()?.stdout, 6, 0, 0, 0, 0, 0, 0)?;

    Ok(())
}
