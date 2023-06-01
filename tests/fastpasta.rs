use crate::util::*;
mod util;

/// Test that all test data files can be parsed successfully
#[test]
fn fastpasta_version() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg("--version").arg("-v2");
    cmd.assert().success();

    match_on_out_no_case(&cmd.output().unwrap().stdout, "fastpasta", 1)?;
    match_on_out_no_case(
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
            predicate::str::contains("Subcommand").and(
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

    cmd.arg(FILE_10_RDH).arg("check").arg("sanity").arg("-v2");
    cmd.assert().success();

    // Take the output of stderr and match it with a pattern once
    match_on_out_no_case(&cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}

#[test]
fn file_exists_exit_successful_err_not_hbf() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_ERR_NOT_HBF)
        .arg("check")
        .arg("sanity")
        .arg("-v2");
    cmd.assert().success();

    match_on_out_no_case(&cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}

#[test]
fn file_exists_exit_successful_thrs_cdw_links() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_THRS_CDW_LINKS)
        .arg("check")
        .arg("sanity")
        .arg("-v2");
    cmd.assert().success();

    match_on_out_no_case(&cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}

#[test]
fn file_exists_exit_successful_readout_superpage_1() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_READOUT_SUPERPAGE_1)
        .arg("check")
        .arg("sanity")
        .arg("-v2");
    cmd.assert().success();

    match_on_out_no_case(&cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}

#[test]
fn file_exists_exit_successful_1_hbf_bad_cdp_structure() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_CDP_STRUCTURE)
        .arg("check")
        .arg("sanity")
        .arg("-v2");
    cmd.assert().success();

    match_on_out_no_case(&cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}

#[test]
fn file_exists_exit_successful_1_hbf_bad_dw_ddw0() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_DW_DDW0)
        .arg("check")
        .arg("sanity")
        .arg("-v2");
    cmd.assert().success();

    match_on_out_no_case(&cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}

#[test]
fn file_exists_exit_successful_1_hbf_bad_ihw_tdh() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_IHW_TDH)
        .arg("check")
        .arg("sanity")
        .arg("-v2");
    cmd.assert().success();

    match_on_out_no_case(&cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}

#[test]
fn file_exists_exit_successful_1_hbf_bad_its_payload() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_ITS_PAYLOAD)
        .arg("check")
        .arg("sanity")
        .arg("-v2");
    cmd.assert().success();

    match_on_out_no_case(&cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}

#[test]
fn file_exists_exit_successful_1_hbf_bad_tdt() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_TDT)
        .arg("check")
        .arg("sanity")
        .arg("-v2");
    cmd.assert().success();

    match_on_out_no_case(&cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}

#[test]
fn file_exists_exit_successful_tdh_no_data() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_TDH_NO_DATA)
        .arg("check")
        .arg("sanity")
        .arg("-v2");
    cmd.assert().success();

    // Take the output of stderr and match it with a pattern once
    match_on_out_no_case(&cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}

#[test]
fn file_exists_exit_successful_tdh_no_data_ihw() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_TDH_NO_DATA_IHW)
        .arg("check")
        .arg("sanity")
        .arg("-v2");
    cmd.assert().success();

    // Take the output of stderr and match it with a pattern once
    match_on_out_no_case(&cmd.output().unwrap().stderr, "exit success", 1)?;

    Ok(())
}

/// Check that a not found file returns a fatal error, with a description of an OS error
///
/// Try with all the different verbosity values 0-4
#[test]
fn file_doesnt_exist_v0() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg("test/file/doesnt/exist")
        .arg("check")
        .arg("sanity")
        .arg("-v0");
    cmd.assert().failure().stderr(
        predicate::str::contains("ERROR - FATAL:").and(predicate::str::contains("os error")),
    );
    Ok(())
}
#[test]
fn file_doesnt_exist_v1() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg("test/file/doesnt/exist")
        .arg("check")
        .arg("sanity")
        .arg("-v1");

    cmd.assert().failure().stderr(
        predicate::str::contains("ERROR - FATAL:").and(predicate::str::contains("os error")),
    );

    Ok(())
}

#[test]
fn file_doesnt_exist_v2() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg("test/file/doesnt/exist")
        .arg("check")
        .arg("sanity")
        .arg("-v2");

    cmd.assert().failure().stderr(
        predicate::str::contains("ERROR - FATAL:").and(predicate::str::contains("os error")),
    );

    Ok(())
}

#[test]
fn file_doesnt_exist_v3() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg("test/file/doesnt/exist")
        .arg("check")
        .arg("sanity")
        .arg("-v3");

    cmd.assert().failure().stderr(
        predicate::str::contains("ERROR - FATAL:").and(predicate::str::contains("os error")),
    );
    Ok(())
}

#[test]
fn file_doesnt_exist_v4() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg("test/file/doesnt/exist")
        .arg("check")
        .arg("sanity")
        .arg("-v4");

    cmd.assert().failure().stderr(
        predicate::str::contains("ERROR - FATAL:").and(predicate::str::contains("os error")),
    );

    Ok(())
}
