use crate::util::*;
mod util;

#[test]
fn check_sanity_its() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_ITS_PAYLOAD)
        .arg("check")
        .arg("sanity")
        .arg("its")
        .arg("-v4");
    cmd.assert().success();

    match_on_out(
        false,
        &cmd.output().unwrap().stderr,
        "error.*0x50:.*id is not",
        1,
    )?;
    match_on_out(false, &cmd.output().unwrap().stdout, "total.*errors.*1", 1)?;

    Ok(())
}

#[test]
fn check_all_its() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_ITS_PAYLOAD)
        .arg("check")
        .arg("all")
        .arg("its")
        .arg("-v4");
    cmd.assert().success();

    match_on_out(
        false,
        &cmd.output().unwrap().stderr,
        "error.*0x50:.*id is not",
        1,
    )?;
    match_on_out(
        false,
        &cmd.output().unwrap().stderr,
        "error.*0x70:.*lane 8.*IHW", // Error with lane 8 not being active according to IHW
        1,
    )?;
    match_on_out(false, &cmd.output().unwrap().stdout, "total.*errors.*2", 1)?;

    Ok(())
}

#[test]
/// Test that the error code filter specifying both the error codes that are reported in this data prints both the expected errors
fn check_all_its_with_error_code_filter_allowing_all_errors(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_ITS_PAYLOAD)
        .arg("check")
        .arg("all")
        .arg("its")
        .arg("-v4")
        .arg("--show-only-errors-with-codes")
        .arg("40")
        .arg("72");
    cmd.assert().success();

    match_on_out(
        false,
        &cmd.output().unwrap().stderr,
        "error.*0x50:.*id is not",
        1,
    )?;
    match_on_out(
        false,
        &cmd.output().unwrap().stderr,
        "error.*0x70:.*lane 8.*IHW", // Error with lane 8 not being active according to IHW
        1,
    )?;
    match_on_out(false, &cmd.output().unwrap().stdout, "total.*errors.*2", 1)?;

    Ok(())
}

#[test]
/// Test that the error code filter specifying only one of the error codes that are reported in this data prints only the expected error
fn check_all_its_with_error_code_filter_one_error() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_ITS_PAYLOAD)
        .arg("check")
        .arg("all")
        .arg("its")
        .arg("-v4")
        .arg("--show-only-errors-with-codes")
        .arg("72");
    cmd.assert().success();
    // Expect 0 matches because this error is filtered out when only showing error code 72
    match_on_out(
        false,
        &cmd.output().unwrap().stderr,
        "error.*0x50:.*id is not",
        0,
    )?;
    // This has error code 72 so it should still be shown
    match_on_out(
        false,
        &cmd.output().unwrap().stderr,
        "error.*0x70:.*lane 8.*IHW", // Error with lane 8 not being active according to IHW
        1,
    )?;
    match_on_out(false, &cmd.output().unwrap().stdout, "total.*errors.*2", 1)?;

    Ok(())
}

#[test]
fn check_all_its_stave() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_ITS_PAYLOAD)
        .arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("-v4")
        .arg("--filter-its-stave")
        .arg("l0_12");
    cmd.assert().success();

    match_on_out(
        false,
        &cmd.output().unwrap().stderr,
        "error.*0x50:.*id is not",
        1,
    )?;
    match_on_out(
        false,
        &cmd.output().unwrap().stderr,
        "error.*0x70:.*lane 8.*IHW", // Error with lane 8 not being active according to IHW
        1,
    )?;
    match_on_out(false, &cmd.output().unwrap().stdout, "total.*errors.*2", 1)?;

    Ok(())
}

#[test]
fn view_rdh() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_ITS_PAYLOAD)
        .arg("view")
        .arg("rdh")
        .arg("-v4");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output().unwrap().stderr)?;
    // match lines that have the RDH version 7, header size 64, and feeid 524
    match_on_out(false, &cmd.output().unwrap().stdout, ".*7.*64.*524", 2)?;

    Ok(())
}

#[test]
fn view_its_readout_frames() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_ITS_PAYLOAD)
        .arg("view")
        .arg("its-readout-frames")
        .arg("-v4");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output().unwrap().stderr)?;
    match_on_out(true, &cmd.output().unwrap().stdout, "RDH", 2)?;
    // It's an error that two IHW IDs appear in a row, but this view should show two IHWs
    match_on_out(true, &cmd.output().unwrap().stdout, "IHW", 2)?;
    match_on_out(true, &cmd.output().unwrap().stdout, "TDT", 1)?;
    match_on_out(true, &cmd.output().unwrap().stdout, "DDW", 1)?;

    Ok(())
}
