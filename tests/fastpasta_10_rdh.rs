use assert_fs::TempDir;
use predicates::str::contains;

use crate::util::*;
mod util;

// Asserts that the end of processing report summary contains correct information
fn validate_report_summary(byte_output: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let match_patterns = vec![
        "Trigger Type.*0x6A03",
        "Trigger Type.*SOC",
        "RDH.*Version.*7",
        "Total.*RDHs.*10",
        "Total.*hbfs.*5",
        "((layers)|(staves)).*((layers)|(staves)).*L0_12",
    ];
    for pattern in match_patterns {
        match_on_out_no_case(byte_output, pattern, 1)?;
    }
    Ok(())
}

#[test]
fn view_rdh() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_10_RDH).arg("view").arg("rdh").arg("-v2");

    cmd.assert()
        .success()
        .stdout(is_match(": .* (7|6) .* 64 .* (0|2)")?.count(10));

    Ok(())
}

#[test]
fn view_hbf() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_10_RDH).arg("view").arg("hbf");
    use predicate::str::contains;
    cmd.assert().success().stdout(
        contains("RDH").count(10).and(
            contains("IHW").count(5).and(
                contains("TDH")
                    .count(5)
                    .and(contains("TDT").count(5).and(contains("DDW").count(5))),
            ),
        ),
    );

    Ok(())
}

#[test]
fn check_sanity() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_10_RDH).arg("check").arg("sanity");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;
    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

#[test]
fn check_sanity_its() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_10_RDH).arg("check").arg("sanity").arg("its");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;
    // Asserts on stdout
    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

#[test]
fn check_all() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_10_RDH).arg("check").arg("all");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;
    // Asserts on stdout
    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

#[test]
fn check_all_its() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_10_RDH).arg("check").arg("all").arg("its");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;

    // Asserts on stdout
    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

#[test]
fn check_all_its_trigger_period_missing_arg() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_10_RDH)
        .arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("--its-trigger-period")
        .arg("1");
    cmd.assert()
        .failure()
        .stderr(contains("arguments were not provided:").and(contains("filter-its-stave")));

    Ok(())
}

#[test]
fn check_all_its_trigger_period_stave_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_10_RDH)
        .arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("-v2")
        .arg("--its-trigger-period")
        .arg("1")
        .arg("--filter-its-stave")
        .arg("L3_2");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;

    match_on_out_no_case(&cmd.output()?.stdout, "its stave.*none.*not found.*l3_2", 1)?;

    Ok(())
}
#[test]
fn check_all_its_trigger_period_mismatch() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_10_RDH)
        .arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("-v3")
        .arg("--its-trigger-period")
        .arg("1")
        .arg("--filter-its-stave")
        .arg("L0_12");
    cmd.assert().success();

    match_on_out_no_case(&cmd.output()?.stderr, "error - ", 4)?;
    match_on_out_no_case(&cmd.output()?.stderr, "warn - ", 0)?;

    match_on_out_no_case(&cmd.output()?.stderr, r"period.*mismatch.*1 !=", 4)?;
    match_on_out_no_case(&cmd.output()?.stdout, "its stave.*l0_12", 1)?;

    Ok(())
}

#[test]
fn check_all_its_trigger_period() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_10_RDH)
        .arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("-v3")
        .arg("--its-trigger-period")
        .arg("0")
        .arg("--filter-its-stave")
        .arg("L0_12");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;

    match_on_out_no_case(&cmd.output()?.stdout, "its stave.*l0_12", 1)?;

    Ok(())
}

#[test]
fn check_all_its_stave_filter() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_10_RDH)
        .arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("-v4")
        .arg("--filter-its-stave")
        .arg("L0_12");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;
    match_on_out_no_case(&cmd.output()?.stdout, "its stave.*l0_12", 1)?;

    Ok(())
}

#[test]
fn check_all_its_stave() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_10_RDH)
        .arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("-v4");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;

    Ok(())
}

#[test]
fn filter_its_stave() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg(FILE_10_RDH)
        .arg("--filter-its-stave")
        .arg("L0_12")
        .arg("-o")
        .arg(FILE_OUTPUT_TMP);

    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;
    // Asserts on stdout
    match_on_out_no_case(&cmd.output()?.stdout, r"filter.*stats", 1)?;

    // Checking the filtered stats
    match_on_out_no_case(&cmd.output()?.stdout, r".*filter.*stats", 1)?;
    match_on_out_no_case(&cmd.output()?.stdout, r"\|.*RDHs.*10", 1)?;

    match_on_out_no_case(&cmd.output()?.stdout, r".*L0_12", 1)?;

    // cleanup temp file
    std::fs::remove_file(FILE_OUTPUT_TMP).expect("Could not remove temp file");

    Ok(())
}

#[test]
fn filter_its_stave_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;
    let stave_to_filter = "L3_0"; // Not in the data
    cmd.arg(FILE_10_RDH)
        .arg("--filter-its-stave")
        .arg(stave_to_filter)
        .arg("-o")
        .arg(FILE_OUTPUT_TMP);

    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;
    // Asserts on stdout
    match_on_out_no_case(&cmd.output()?.stdout, "Total.*RDHs.*10", 1)?;
    // Checking the filtered stats
    match_on_out_no_case(&cmd.output()?.stdout, r".*filter.*stats", 1)?;
    match_on_out_no_case(&cmd.output()?.stdout, r"\|.* RDHs.*0", 1)?;

    match_on_out_no_case(
        &cmd.output()?.stdout,
        &(r".*not found:.*".to_string() + stave_to_filter),
        1,
    )?;

    // cleanup temp file
    std::fs::remove_file(FILE_OUTPUT_TMP).expect("Could not remove temp file");

    Ok(())
}

#[test]
fn filter_fee() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;
    let fee_id_to_filter = "524";
    cmd.arg(FILE_10_RDH)
        .arg("--filter-fee")
        .arg(fee_id_to_filter)
        .arg("-o")
        .arg(FILE_OUTPUT_TMP);

    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;
    // Asserts on stdout
    match_on_out_no_case(&cmd.output()?.stdout, "Total.*RDHs.*10", 1)?;
    // Checking the filtered stats
    match_on_out_no_case(&cmd.output()?.stdout, r".*filter.*stats", 1)?;
    match_on_out_no_case(
        &cmd.output()?.stdout,
        &(r"FEE ID.*".to_string() + fee_id_to_filter),
        // Expect 2 occurences, one in the global stats and one in the filtered stats
        2,
    )?;

    match_on_out_no_case(&cmd.output()?.stdout, r"\|.* RDHs.*10", 1)?;

    // cleanup temp file
    std::fs::remove_file(FILE_OUTPUT_TMP).expect("Could not remove temp file");

    Ok(())
}

#[test]
fn filter_fee_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;
    let fee_id_to_filter = "1337";
    cmd.arg(FILE_10_RDH)
        .arg("--filter-fee")
        .arg(fee_id_to_filter)
        .arg("-o")
        .arg(FILE_OUTPUT_TMP);

    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;
    // Asserts on stdout
    match_on_out_no_case(&cmd.output()?.stdout, "Total.*RDHs.*10", 1)?;
    // Checking the filtered stats
    match_on_out_no_case(&cmd.output()?.stdout, r".*filter.*stats", 1)?;
    match_on_out_no_case(
        &cmd.output()?.stdout,
        &(r"FEE.*not found.*".to_string() + fee_id_to_filter),
        1,
    )?;

    match_on_out_no_case(&cmd.output()?.stdout, r"\|.* RDHs.* 0 ", 1)?;

    // cleanup temp file
    std::fs::remove_file(FILE_OUTPUT_TMP).expect("Could not remove temp file");

    Ok(())
}

#[test]
fn view_its_readout_frame() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_10_RDH).arg("view").arg("its-readout-frames");
    use predicate::str::contains;
    cmd.assert().success().stdout(
        contains("RDH").count(10).and(
            contains("IHW").count(5).and(
                contains("TDH")
                    .count(5)
                    .and(contains("TDT").count(5).and(contains("DDW").count(5))),
            ),
        ),
    );

    Ok(())
}

#[test]
fn view_its_readout_frame_data() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_10_RDH)
        .arg("view")
        .arg("its-readout-frames-data");
    use predicate::str::contains;
    cmd.assert().success().stdout(
        contains("RDH").count(10).and(
            contains("IHW").count(5).and(
                contains("TDH")
                    .count(5)
                    .and(contains("TDT").count(5).and(contains("DDW").count(5))),
            ),
        ),
    );

    // 3 data lanes, expect to see data from all of them 5 times.
    match_on_out_no_case(&cmd.output()?.stdout, "data.*26]", 5)?;
    match_on_out_no_case(&cmd.output()?.stdout, "data.*27]", 5)?;
    match_on_out_no_case(&cmd.output()?.stdout, "data.*28]", 5)?;

    Ok(())
}

#[test]
fn check_sanity_stdin() -> Result<(), Box<dyn std::error::Error>> {
    use assert_cmd::cmd::*;
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.pipe_stdin(FILE_10_RDH)?.arg("check").arg("sanity");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;
    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

#[test]
fn filter_link_check_sanity_pipe_between_fastpasta() -> Result<(), Box<dyn std::error::Error>> {
    // Test piping to fastpasta and writing to stdout and piping into another fastpasta instance
    let mut cmd1 = Command::cargo_bin("fastpasta")?;

    // Pipe 10_rdh.raw into fastpasta and filter link 8
    cmd1.pipe_stdin(FILE_10_RDH)?.arg("--filter-link").arg("8");
    // Confirm successful execution and copy output
    let out = cmd1.assert().success().get_output().stdout.clone();
    assert_no_errors_or_warn(&cmd1.output()?.stderr)?;

    // Pipe the output of the first fastpasta instance into another fastpasta instance
    let mut cmd2 = Command::cargo_bin("fastpasta")?;
    cmd2.write_stdin(out).arg("check").arg("sanity");

    // Confirm successful execution and validate the report summary and that there was no errors
    cmd2.assert().success();
    assert_no_errors_or_warn(&cmd2.output()?.stderr)?;
    validate_report_summary(&cmd2.output()?.stdout)?;

    Ok(())
}

#[test]
fn check_sanity_custom_exit_code() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_10_RDH)
        .arg("check")
        .arg("sanity")
        .arg("-E")
        .arg("1");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;
    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

#[test]
fn check_sanity_custom_checks_cdp_count() -> Result<(), Box<dyn std::error::Error>> {
    let custom_checks_str = r#"
# Number of CRU Data Packets expected in the data
# Example value: 20 [type: u32]
cdps = 10
# Number of Physics (PhT) Triggers expected in the data
# Example value: 20 [type: u32]
#triggers_pht = (uncomment and set to enable)
"#;
    let custom_checks_file_name = "tmp_custom_checks.toml";
    let tmp_dir = TempDir::new()?;
    let tmp_custom_checks_path = tmp_dir.path().join(custom_checks_file_name);

    let mut custom_checks_file = std::fs::File::create(tmp_custom_checks_path.clone())?;
    std::io::Write::write_all(&mut custom_checks_file, custom_checks_str.as_bytes())?;

    custom_checks_file.sync_all()?;

    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg(FILE_10_RDH)
        .arg("check")
        .arg("sanity")
        .arg("--checks-toml")
        .arg(tmp_custom_checks_path);

    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;
    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

#[test]
fn check_sanity_custom_checks_bad_cdp_count() -> Result<(), Box<dyn std::error::Error>> {
    let custom_checks_str = r#"
# Number of CRU Data Packets expected in the data
# Example value: 20 [type: u32]
cdps = 0
# Number of Physics (PhT) Triggers expected in the data
# Example value: 20 [type: u32]
#triggers_pht = (uncomment and set to enable)
"#;
    let custom_checks_file_name = "tmp_custom_checks.toml";
    let tmp_dir = TempDir::new()?;
    let tmp_custom_checks_path = tmp_dir.path().join(custom_checks_file_name);

    let mut custom_checks_file = std::fs::File::create(tmp_custom_checks_path.clone())?;
    std::io::Write::write_all(&mut custom_checks_file, custom_checks_str.as_bytes())?;

    custom_checks_file.sync_all()?;

    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg(FILE_10_RDH)
        .arg("check")
        .arg("sanity")
        .arg("--checks-toml")
        .arg(tmp_custom_checks_path);

    cmd.assert().success();

    // There's 10 CDPs in the file, but the custom checks file expects 0
    match_on_out_no_case(&cmd.output()?.stderr, "ERROR.*expect.*0.*found.*10", 1)?;
    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

#[test]
fn check_sanity_custom_checks_trigger_pht_count() -> Result<(), Box<dyn std::error::Error>> {
    let custom_checks_str = r#"
# Number of CRU Data Packets expected in the data
# Example value: 20 [type: u32]
# cdps = 10
# Number of Physics (PhT) Triggers expected in the data
# Example value: 20 [type: u32]
triggers_pht = 0 # PhT triggers are only expected in triggered mode
"#;
    let custom_checks_file_name = "tmp_custom_checks.toml";
    let tmp_dir = TempDir::new()?;
    let tmp_custom_checks_path = tmp_dir.path().join(custom_checks_file_name);

    let mut custom_checks_file = std::fs::File::create(tmp_custom_checks_path.clone())?;
    std::io::Write::write_all(&mut custom_checks_file, custom_checks_str.as_bytes())?;

    custom_checks_file.sync_all()?;

    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg(FILE_10_RDH)
        .arg("check")
        .arg("sanity")
        .arg("--checks-toml")
        .arg(tmp_custom_checks_path);

    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;
    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

#[test]
fn check_sanity_custom_checks_rdh_version() -> Result<(), Box<dyn std::error::Error>> {
    let custom_checks_str = r#"
rdh_version = 7
"#;
    let custom_checks_file_name = "tmp_custom_checks.toml";
    let tmp_dir = TempDir::new()?;
    let tmp_custom_checks_path = tmp_dir.path().join(custom_checks_file_name);

    let mut custom_checks_file = std::fs::File::create(tmp_custom_checks_path.clone())?;
    std::io::Write::write_all(&mut custom_checks_file, custom_checks_str.as_bytes())?;

    custom_checks_file.sync_all()?;

    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg(FILE_10_RDH)
        .arg("check")
        .arg("sanity")
        .arg("--checks-toml")
        .arg(tmp_custom_checks_path);

    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;
    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

#[test]
fn check_sanity_custom_checks_rdh_version_wrong() -> Result<(), Box<dyn std::error::Error>> {
    let custom_checks_str = r#"
rdh_version = 6
"#;
    let custom_checks_file_name = "tmp_custom_checks.toml";
    let tmp_dir = TempDir::new()?;
    let tmp_custom_checks_path = tmp_dir.path().join(custom_checks_file_name);

    let mut custom_checks_file = std::fs::File::create(tmp_custom_checks_path.clone())?;
    std::io::Write::write_all(&mut custom_checks_file, custom_checks_str.as_bytes())?;

    custom_checks_file.sync_all()?;

    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg(FILE_10_RDH)
        .arg("check")
        .arg("sanity")
        .arg("--checks-toml")
        .arg(tmp_custom_checks_path);

    cmd.assert().success();

    match_on_out_no_case(&cmd.output()?.stderr, "ERROR -.*rdh.*sanity.*fail", 10)?;
    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}
