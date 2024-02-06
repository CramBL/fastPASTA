use crate::util::*;
use predicates::str::{contains, is_match};
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

    assert_alpide_stats_report(&cmd.output()?.stdout, 15, 0, 0, 0, 0, 0, 0)?;

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
    assert_alpide_stats_report(&cmd.output()?.stdout, 15, 0, 0, 0, 0, 0, 0)?;

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
    assert_alpide_stats_report(&cmd.output()?.stdout, 15, 0, 0, 0, 0, 0, 0)?;

    Ok(())
}

#[test]
fn filter_its_stave() -> Result<(), Box<dyn std::error::Error>> {
    let (_tmp_dir, tmp_fpath) = make_tmp_dir_w_fpath();
    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg(FILE_10_RDH)
        .arg("--filter-its-stave")
        .arg("L0_12")
        .arg("-o")
        .arg(tmp_fpath.as_os_str());

    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;
    // Asserts on stdout
    match_on_out_no_case(&cmd.output()?.stdout, r"filter.*stats", 1)?;

    // Checking the filtered stats
    match_on_out_no_case(&cmd.output()?.stdout, r".*filter.*stats", 1)?;
    match_on_out_no_case(&cmd.output()?.stdout, r"\|.*RDHs.*10", 1)?;

    match_on_out_no_case(&cmd.output()?.stdout, r".*L0_12", 1)?;

    Ok(())
}

#[test]
fn filter_its_stave_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let (_tmp_dir, tmp_fpath) = make_tmp_dir_w_fpath();

    let mut cmd = Command::cargo_bin("fastpasta")?;
    let stave_to_filter = "L3_0"; // Not in the data
    cmd.arg(FILE_10_RDH)
        .arg("--filter-its-stave")
        .arg(stave_to_filter)
        .arg("-o")
        .arg(tmp_fpath.as_os_str());

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

    Ok(())
}

#[test]
fn filter_fee() -> Result<(), Box<dyn std::error::Error>> {
    let (_tmp_dir, tmp_fpath) = make_tmp_dir_w_fpath();

    let mut cmd = Command::cargo_bin("fastpasta")?;
    let fee_id_to_filter = "524";
    cmd.arg(FILE_10_RDH)
        .arg("--filter-fee")
        .arg(fee_id_to_filter)
        .arg("-o")
        .arg(tmp_fpath.as_os_str());

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

    Ok(())
}

#[test]
fn filter_fee_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let (_tmp_dir, tmp_fpath) = make_tmp_dir_w_fpath();

    let mut cmd = Command::cargo_bin("fastpasta")?;
    let fee_id_to_filter = "1337";
    cmd.arg(FILE_10_RDH)
        .arg("--filter-fee")
        .arg(fee_id_to_filter)
        .arg("-o")
        .arg(tmp_fpath.as_os_str());

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

    Ok(())
}

#[test]
fn view_its_readout_frame() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_10_RDH).arg("view").arg("its-readout-frames");
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

    create_custom_checks_toml(custom_checks_str, &tmp_custom_checks_path)?;

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

    create_custom_checks_toml(custom_checks_str, &tmp_custom_checks_path)?;

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

    create_custom_checks_toml(custom_checks_str, &tmp_custom_checks_path)?;

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
    create_custom_checks_toml(custom_checks_str, &tmp_custom_checks_path)?;

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
    // TempDir and Path has to be created in this scope.
    let tmp_dir = TempDir::new()?;
    let tmp_custom_checks_path = tmp_dir.path().join("tmp_custom_checks.toml");

    let custom_checks_str = r#"
rdh_version = 6
"#;
    create_custom_checks_toml(custom_checks_str, &tmp_custom_checks_path)?;

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

#[test]
fn check_sanity_output_stats_json_toml() -> Result<(), Box<dyn std::error::Error>> {
    let (_tmp_dir, tmp_fpath) = make_tmp_dir_w_fpath();

    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg(FILE_10_RDH)
        .arg("check")
        .arg("sanity")
        .arg("--output-stats")
        .arg(tmp_fpath.as_os_str())
        .arg("--stats-format")
        .arg("json");

    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;
    validate_report_summary(&cmd.output()?.stdout)?;

    let stats_str = std::fs::read_to_string(tmp_fpath)?;
    let stats_from_json: fastpasta::stats::stats_collector::StatsCollector =
        serde_json::from_str(&stats_str)?;
    assert_eq!(stats_from_json.rdh_stats().rdh_version(), 7);
    assert_eq!(stats_from_json.rdhs_seen(), 10);

    // Serialize it to TOML and back to a StatsCollector from TOML to compare
    let stats_from_toml: fastpasta::stats::stats_collector::StatsCollector =
        toml::from_str(&toml::to_string(&stats_from_json).unwrap())?;
    assert_eq!(stats_from_json, stats_from_toml);

    // Run the command again with TOML output and compare the output
    let (_tmp_dir2, tmp_fpath2) = make_tmp_dir_w_fpath();
    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg(FILE_10_RDH)
        .arg("check")
        .arg("sanity")
        .arg("--output-stats")
        .arg(tmp_fpath2.as_os_str())
        .arg("--stats-format")
        .arg("toml");

    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;
    validate_report_summary(&cmd.output()?.stdout)?;

    let stats_str = std::fs::read_to_string(tmp_fpath2)?;
    let stats_from_toml: fastpasta::stats::stats_collector::StatsCollector =
        toml::from_str(&stats_str)?;

    assert_eq!(stats_from_json, stats_from_toml);

    Ok(())
}

#[test]
fn check_all_its_stave_output_stats_json_toml() -> Result<(), Box<dyn std::error::Error>> {
    let check_arg = ["check", "all", "its-stave"];

    let (_tmp_dir, tmp_fpath) = make_tmp_dir_w_fpath();

    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg(FILE_10_RDH)
        .args(check_arg)
        .arg("--output-stats")
        .arg(tmp_fpath.as_os_str())
        .arg("--stats-format")
        .arg("json");

    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;
    validate_report_summary(&cmd.output()?.stdout)?;

    let stats_str = std::fs::read_to_string(tmp_fpath)?;
    let stats_from_json: fastpasta::stats::stats_collector::StatsCollector =
        serde_json::from_str(&stats_str)?;
    assert_eq!(stats_from_json.rdh_stats().rdh_version(), 7);
    assert_eq!(stats_from_json.rdhs_seen(), 10);
    assert_eq!(stats_from_json.err_count(), 0);
    assert_eq!(stats_from_json.rdh_stats().trigger_stats().pht(), 0);

    // Serialize it to TOML and back to a StatsCollector from TOML to compare
    let stats_from_toml: fastpasta::stats::stats_collector::StatsCollector =
        toml::from_str(&toml::to_string(&stats_from_json).unwrap())?;
    assert_eq!(stats_from_json, stats_from_toml);

    // Run the command again with TOML output and compare the output
    let (_tmp_dir2, tmp_fpath2) = make_tmp_dir_w_fpath();
    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg(FILE_10_RDH)
        .args(check_arg)
        .arg("--output-stats")
        .arg(tmp_fpath2.as_os_str())
        .arg("--stats-format")
        .arg("TOML");

    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;
    validate_report_summary(&cmd.output()?.stdout)?;

    let stats_str = std::fs::read_to_string(tmp_fpath2)?;
    let stats_from_toml: fastpasta::stats::stats_collector::StatsCollector =
        toml::from_str(&stats_str)?;

    assert_eq!(stats_from_json, stats_from_toml);

    Ok(())
}

#[test]
fn test_check_all_its_with_stats_validation() -> Result<(), Box<dyn std::error::Error>> {
    let check_arg = ["check", "all", "its"];

    let (_tmp_dir, tmp_fpath) = make_tmp_dir_w_named_file("out-stats.json");

    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg(FILE_10_RDH)
        .args(check_arg)
        .arg("--output-stats")
        .arg(tmp_fpath.as_os_str())
        .arg("--stats-format")
        .arg("json");

    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;

    // Now run again with the created stats as input
    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg(FILE_10_RDH)
        .args(check_arg)
        .arg("--input-stats")
        .arg(tmp_fpath.as_os_str());
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;

    // Now alter the stats and run again, expect an error
    let stats_str = std::fs::read_to_string(tmp_fpath)?;
    // alter the json string before deserializing
    let new_wrong_stat_str = stats_str.replace("\"rdhs_seen\": 10", "\"rdhs_seen\": 11");
    assert_ne!(stats_str, new_wrong_stat_str); // make sure we actually changed something
    let stats_from_json: fastpasta::stats::stats_collector::StatsCollector =
        serde_json::from_str(&new_wrong_stat_str)?;

    let (_tmp_dir2, tmp_fpath2) = make_tmp_dir_w_named_file("out-stats-wrong.json");
    let wrong_stats_str = serde_json::to_string(&stats_from_json)?;
    std::fs::write(tmp_fpath2.as_os_str(), wrong_stats_str)?;

    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg(FILE_10_RDH)
        .args(check_arg)
        .arg("--input-stats")
        .arg(tmp_fpath2.as_os_str())
        .arg("--any-errors-exit-code")
        .arg("123");

    cmd.assert().failure().code(123);

    match_on_out_no_case(&cmd.output()?.stderr, "ERROR -.* mismatch.*11", 1)?;

    Ok(())
}

// https://gitlab.cern.ch/mkonig/fastpasta/-/issues/45
// Test that everything up until a faulty payload reading (caused by a faulty RDH offset_to_next field) is processed correctly
#[test]
fn view_its_readout_frames_cutoff_last_byte_padding_issue45(
) -> Result<(), Box<dyn std::error::Error>> {
    // Read the file into a buffer and discard the last byte
    let mut file = std::fs::File::open(FILE_10_RDH)?;
    let mut buffer = Vec::new();
    std::io::Read::read_to_end(&mut file, &mut buffer)?;
    _ = buffer.pop();

    // Make a tmp file and write the buffer to it
    let (_tmp_dir, tmp_file) = make_tmp_dir_w_fpath();
    tmp_file.write_binary(&buffer)?;

    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.pipe_stdin(tmp_file)?
        .arg("view")
        .arg("its-readout-frames");

    // Expect 10 RDHs but the 10th RDHs payload is read incorrectly and is not processed
    // The last payload is just a DDW
    cmd.assert().success().stdout(
        contains("RDH").count(10).and(
            contains("IHW").count(5).and(
                contains("TDH")
                    .count(5)
                    .and(contains("TDT").count(5).and(contains("DDW").count(4))),
            ),
        ),
    );

    Ok(())
}

// https://gitlab.cern.ch/mkonig/fastpasta/-/issues/45
// Test that the erroneous read of the last RDHs payload is detected
#[test]
fn check_sanity_issue45() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = std::fs::File::open(FILE_10_RDH)?;
    let mut buffer = Vec::new();
    std::io::Read::read_to_end(&mut file, &mut buffer)?;
    _ = buffer.pop();

    // Make a tmp file and write the buffer to it
    let (_tmp_dir, tmp_file) = make_tmp_dir_w_fpath();
    tmp_file.write_binary(&buffer)?;

    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.pipe_stdin(tmp_file)?.arg("check").arg("sanity");

    validate_report_summary(&cmd.output()?.stdout)?;
    match_on_out_no_case(&cmd.output()?.stderr, "ERROR - 0x4B0.*payload", 1)?;

    Ok(())
}

/// Checks that now you can supply the path to a file in the position where a subcmd (target system) would otherwise be expected
///
/// This was not possible earlier, where an error would be raised "<path> not a valid command expected <target system variants>"
#[test]
fn check_sanity_path_instead_of_subcmd() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg("check").arg("sanity").arg(FILE_10_RDH);
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;
    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

/// Checks that now you can supply the path to a file in the position where a subcmd (target system) would otherwise be expected
///
/// This was not possible earlier, where an error would be raised "<path> not a valid command expected <target system variants>"
#[test]
fn check_all_path_instead_of_subcmd() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg("check").arg("all").arg(FILE_10_RDH);
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;
    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}
