use predicates::str::contains;

use crate::util::*;
mod util;

#[test]
fn check_sanity_its() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_THRS_CDW_LINKS)
        .arg("check")
        .arg("sanity")
        .arg("its")
        .arg("-v4");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;
    match_on_out_no_case(&cmd.output()?.stdout, "total errors.*0", 1)?;
    match_on_out_no_case(&cmd.output()?.stdout, "trigger type.*0x4893.*SOT", 1)?;
    match_on_out_no_case(&cmd.output()?.stdout, "total rdhs.*6", 1)?;
    match_on_out_no_case(&cmd.output()?.stdout, "links.*8, 9, 11", 1)?;
    match_on_out_no_case(&cmd.output()?.stdout, "hbfs.*3", 1)?;
    match_on_out_no_case(&cmd.output()?.stdout, "layers.*l0_12", 1)?;

    Ok(())
}

#[test]
fn check_all_its() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_THRS_CDW_LINKS)
        .arg("check")
        .arg("all")
        .arg("its")
        .arg("-v4");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;
    match_on_out_no_case(&cmd.output()?.stdout, "total errors.*0", 1)?;

    Ok(())
}

#[test]
fn check_all_its_stave() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_THRS_CDW_LINKS)
        .arg("check")
        .arg("all")
        .arg("its_stave")
        .arg("-v4")
        .arg("--filter-its-stave")
        .arg("L0_12");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;
    match_on_out_no_case(&cmd.output()?.stdout, "total errors.*0", 1)?;

    Ok(())
}

#[test]
fn check_all_its_stave_trigger_period() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_THRS_CDW_LINKS)
        .arg("check")
        .arg("all")
        .arg("its_stave")
        .arg("-v4")
        .arg("--filter-its-stave")
        .arg("L0_12")
        .arg("--its-trigger-period")
        .arg("1337"); // There's no TDHs with internal trigger set in the data
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;
    match_on_out_no_case(&cmd.output()?.stdout, "total errors.*0", 1)?;

    Ok(())
}

#[test]
fn view_its_readout_frames() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_THRS_CDW_LINKS)
        .arg("view")
        .arg("its-readout-frames")
        .arg("-v4");

    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;

    cmd.assert().stdout(contains("RDH").count(6));

    Ok(())
}
