use crate::util::*;
mod util;

#[test]
fn check_sanity_its() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_IHW_TDH)
        .arg("check")
        .arg("sanity")
        .arg("its")
        .arg("-v4");
    cmd.assert().success();

    match_on_out_no_case(&cmd.output().unwrap().stderr, "error.*0x40:", 1)?;
    match_on_out_no_case(&cmd.output().unwrap().stderr, "error.*0x50:", 1)?;
    match_on_out_no_case(&cmd.output().unwrap().stdout, "total.*errors.*2", 1)?;

    Ok(())
}

#[test]
fn check_all_its() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_IHW_TDH)
        .arg("check")
        .arg("all")
        .arg("its")
        .arg("-v4");
    cmd.assert().success();

    match_on_out_no_case(&cmd.output().unwrap().stderr, "error.*0x40:", 1)?;
    match_on_out_no_case(&cmd.output().unwrap().stderr, "error.*0x50:", 1)?;
    match_on_out_no_case(&cmd.output().unwrap().stdout, "total.*errors.*2", 1)?;

    Ok(())
}

#[test]
fn check_all_its_stave() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_IHW_TDH)
        .arg("check")
        .arg("all")
        .arg("its_stave")
        .arg("-v4")
        .arg("--filter-its-stave")
        .arg("l0_12");
    cmd.assert().success();

    match_on_out_no_case(&cmd.output().unwrap().stderr, "error.*0x40:", 1)?;
    match_on_out_no_case(&cmd.output().unwrap().stderr, "error.*0x50:", 1)?;
    match_on_out_no_case(&cmd.output().unwrap().stdout, "total.*errors.*2", 1)?;

    Ok(())
}

#[test]
fn view_rdh() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_IHW_TDH)
        .arg("view")
        .arg("rdh")
        .arg("-v4");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output().unwrap().stderr)?;
    // match lines that have the RDH version 7, header size 64, and feeid 524
    match_on_out_no_case(&cmd.output().unwrap().stdout, ".*7.*64.*524", 2)?;

    Ok(())
}

#[test]
fn view_its_readout_frames() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_IHW_TDH)
        .arg("view")
        .arg("its-readout-frames")
        .arg("-v4");
    cmd.assert().success();

    match_on_out_no_case(&cmd.output().unwrap().stdout, ": RDH", 2)?;
    match_on_out_no_case(&cmd.output().unwrap().stdout, ": DDW", 1)?;
    match_on_out_no_case(&cmd.output().unwrap().stdout, ": TDT", 1)?;

    Ok(())
}
