use crate::util::*;
mod util;

/// Test that all test data files can be parsed successfully
#[test]
fn fastpasta_version() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg("--version").arg("-v2");
    cmd.assert().success();

    match_on_out(false, &cmd.output().unwrap().stdout, "fastpasta", 1)?;
    match_on_out(
        false,
        &cmd.output().unwrap().stdout,
        r"fastpasta.*[1-9]{1,2}\.[0-9]{1,4}\.[0-9]{1,10}", // Match first number from 1-9 as major version 1 is already out
        1,
    )?;

    Ok(())
}

#[test]
fn fastpasta_help() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    // Long help
    cmd.arg("--help").arg("-v2");
    cmd.assert().success().stdout(
        predicate::str::contains("Arguments").and(
            predicate::str::contains("Enable check mode").and(
                predicate::str::contains("Options").and(
                    predicate::str::contains("Commands").and(
                        predicate::str::contains("Usage")
                            .and(predicate::str::contains("help"))
                            .and(predicate::str::contains("version")),
                    ),
                ),
            ),
        ),
    );

    Ok(())
}

/// Test that all test data files can be parsed successfully
#[test]
fn file_exists_exit_successful_10_rdh() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_10_RDH).arg("check").arg("sanity").arg("-v4");
    cmd.assert().success();

    // Take the output of stderr and match it with a pattern once
    match_on_out(false, &cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}

#[test]
fn file_exists_exit_successful_err_not_hbf() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_ERR_NOT_HBF)
        .arg("check")
        .arg("sanity")
        .arg("-v4");
    cmd.assert().success();

    match_on_out(false, &cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}

#[test]
fn file_exists_exit_successful_thrs_cdw_links() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_THRS_CDW_LINKS)
        .arg("check")
        .arg("sanity")
        .arg("-v4");
    cmd.assert().success();

    match_on_out(false, &cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}

#[test]
fn file_exists_exit_successful_readout_superpage_1() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_READOUT_SUPERPAGE_1)
        .arg("check")
        .arg("sanity")
        .arg("-v4");
    cmd.assert().success();

    match_on_out(false, &cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}

#[test]
fn file_exists_exit_successful_1_hbf_bad_cdp_structure() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_CDP_STRUCTURE)
        .arg("check")
        .arg("sanity")
        .arg("-v4");
    cmd.assert().success();

    match_on_out(false, &cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}

#[test]
fn file_exists_exit_successful_1_hbf_bad_dw_ddw0() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_DW_DDW0)
        .arg("check")
        .arg("sanity")
        .arg("-v4");
    cmd.assert().success();

    match_on_out(false, &cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}

#[test]
fn file_exists_exit_successful_1_hbf_bad_ihw_tdh() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_IHW_TDH)
        .arg("check")
        .arg("sanity")
        .arg("-v4");
    cmd.assert().success();

    match_on_out(false, &cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}

#[test]
fn file_exists_exit_successful_1_hbf_bad_its_payload() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_ITS_PAYLOAD)
        .arg("check")
        .arg("sanity")
        .arg("-v4");
    cmd.assert().success();

    match_on_out(false, &cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}

#[test]
fn file_exists_exit_successful_1_hbf_bad_tdt() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_TDT)
        .arg("check")
        .arg("sanity")
        .arg("-v4");
    cmd.assert().success();

    match_on_out(false, &cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}

#[test]
fn file_exists_exit_successful_tdh_no_data() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_TDH_NO_DATA)
        .arg("check")
        .arg("sanity")
        .arg("-v4");
    cmd.assert().success();

    // Take the output of stderr and match it with a pattern once
    match_on_out(false, &cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}

#[test]
fn file_exists_exit_successful_tdh_no_data_ihw() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_TDH_NO_DATA_IHW)
        .arg("check")
        .arg("sanity")
        .arg("-v4");
    cmd.assert().success();

    // Take the output of stderr and match it with a pattern once
    match_on_out(false, &cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}

#[test]
fn file_exists_exit_successful_rawtf_epn180_l6_1() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_RAWTF_EPN180_L6_1)
        .arg("check")
        .arg("sanity")
        .arg("-v4");
    cmd.assert().success();

    // Take the output of stderr and match it with a pattern once
    match_on_out(false, &cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}

#[test]
fn file_exists_exit_successful_rawtf_fee_24612_4rdhs() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_RAWTF_FEE_24612_4RDHS)
        .arg("check")
        .arg("sanity")
        .arg("-v4");
    cmd.assert().success();

    // Take the output of stderr and match it with a pattern once
    match_on_out(false, &cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}
#[test]
fn file_exists_exit_successful_invalid_lane_order_1rdh() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_INVALID_LANE_ORDER_1HBF)
        .arg("check")
        .arg("sanity")
        .arg("-v4");
    cmd.assert().success();

    // Take the output of stderr and match it with a pattern once
    match_on_out(false, &cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}

#[test]
fn file_exists_exit_successful_ci_ols_data_1hbf() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_CI_OLS_DATA_1HBF)
        .arg("check")
        .arg("sanity")
        .arg("-v4");
    cmd.assert().success();

    // Take the output of stderr and match it with a pattern once
    match_on_out(false, &cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}

#[test]
fn file_exists_exit_successful_2_rdh_det_field_v1_21_0() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_2_RDH_DET_FIELD_V1_21_0)
        .arg("check")
        .arg("sanity")
        .arg("-v4");
    cmd.assert().success();

    // Take the output of stderr and match it with a pattern once
    match_on_out(false, &cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}

#[test]
fn file_exists_exit_successful_2_hbf_2nd_bad_frame() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_2_HBF_2ND_BAD_FRAME)
        .arg("check")
        .arg("sanity")
        .arg("-v4");
    cmd.assert().success();

    // Take the output of stderr and match it with a pattern once
    match_on_out(false, &cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}

#[test]
fn file_exists_exit_successful_12_links_2hbf() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_12_LINKS_2HBF)
        .arg("check")
        .arg("sanity")
        .arg("-v4");
    cmd.assert().success();

    // Take the output of stderr and match it with a pattern once
    match_on_out(false, &cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}
