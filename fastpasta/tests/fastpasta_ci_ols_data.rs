use crate::util::*;
mod util;

// Asserts that the end of processing report summary contains correct information
fn validate_report_summary(byte_output: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let match_patterns = vec![
        "Trigger Type.*0x4893",
        "RDH.*Version.*7",
        "Total.*RDHs.*2",
        "Total.*hbfs.*1",
        "((layers)|(staves)).*((layers)|(staves)).*L5_42",
    ];
    for pattern in match_patterns {
        match_on_out_no_case(byte_output, pattern, 1)?;
    }

    Ok(())
}

#[test]
fn check_sanity() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_CI_OLS_DATA_1HBF).arg("check").arg("sanity");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;

    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

#[test]
// Test with error code filter, should have no effect as there are no errors
// But still a scenario that needs a test
fn check_sanity_with_error_code_filter() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_CI_OLS_DATA_1HBF)
        .arg("check")
        .arg("sanity")
        .args(["--show-only-errors-with-codes", "10", "0", "200"]);
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;

    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

#[test]
fn check_all_its_stave_custom_chip_id_order_errors() -> Result<(), Box<dyn std::error::Error>> {
    // Check that errors are present when a custom check on the chip id order is used
    // The trigger count is also wrong and will cause an error
    let custom_checks_str = r#"
# Number of CRU Data Packets expected in the data
# Example value: 20 [type: u32]
# cdps = 10
# Number of Physics (PhT) Triggers expected in the data
# Example value: 20 [type: u32]
triggers_pht = 0 # PhT triggers are only expected in triggered mode
chip_orders_ob = [[0, 1, 2, 3, 4, 5, 6], [8, 9, 10, 11, 12, 13, 14]]
"#;
    let custom_checks_file_name = "check_all_its_stave_custom_chip_id_order_errors.toml";
    let tmp_dir = assert_fs::TempDir::new()?;
    let tmp_custom_checks_path = tmp_dir.path().join(custom_checks_file_name);
    let mut custom_checks_file = std::fs::File::create(tmp_custom_checks_path.clone())?;
    std::io::Write::write_all(&mut custom_checks_file, custom_checks_str.as_bytes())?;

    custom_checks_file.sync_all()?;

    let (_tmp_dir, tmp_fpath_stats) = make_tmp_dir_w_named_file("out-stats.toml");

    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg(FILE_CI_OLS_DATA_1HBF)
        .arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("--checks-toml")
        .arg(&tmp_custom_checks_path)
        .arg("--output-final-stats")
        .arg(tmp_fpath_stats.as_os_str())
        .arg("--stats-data-format")
        .arg("toml");

    cmd.assert().success();

    match_on_out_no_case(&cmd.output()?.stderr, r"\[E9005\] chip id order", 1)?;
    match_on_out_no_case(&cmd.output()?.stderr, "error.*expected.*PhT trigger", 1)?;

    let stats_toml = read_stats_from_file(&tmp_fpath_stats, "toml")?;

    assert_eq!(stats_toml.err_count(), 2);
    assert_eq!(stats_toml.rdhs_seen(), 2);
    assert_eq!(stats_toml.hbfs_seen(), 1);
    assert_eq!(stats_toml.payload_size(), 480);
    assert_eq!(stats_toml.rdh_stats().trigger_stats().pht(), 2);
    assert_eq!(stats_toml.rdh_stats().trigger_stats().hb(), 2);
    assert_eq!(stats_toml.rdh_stats().trigger_stats().orbit(), 2);

    // Feed the stats back and check that it matches the collected stats
    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg(FILE_CI_OLS_DATA_1HBF)
        .arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("--checks-toml")
        .arg(tmp_custom_checks_path)
        .arg("--input-stats")
        .arg(tmp_fpath_stats.as_os_str());

    cmd.assert().success();
    // No errors mentioning a mismatch between the expected and the collected stats
    match_on_out_no_case(&cmd.output()?.stderr, r"ERROR - .*mismatch", 0)?;

    // Now run it back without the custom checks, now there should be no errors in the data processing
    // But there's now a mismatch between the expected and the collected stats as errors are expected
    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg(FILE_CI_OLS_DATA_1HBF)
        .arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("--input-stats")
        .arg(tmp_fpath_stats.as_os_str());

    cmd.assert().success();

    match_on_out_no_case(
        &cmd.output()?.stderr,
        r"ERROR - .*total_errors.*mismatch",
        1,
    )?;

    Ok(())
}

#[test]
fn check_all_its_stave_custom_chip_id_order_errors_adjusted_toml(
) -> Result<(), Box<dyn std::error::Error>> {
    // Check that errors are now gone when the faulty chip ordering
    //  due to a disabled ALPIDE chip is added to the valid ones in the TOML
    let custom_checks_str = r#"
# Number of CRU Data Packets expected in the data
# Example value: 20 [type: u32]
# cdps = 10
# Number of Physics (PhT) Triggers expected in the data
# Example value: 20 [type: u32]
triggers_pht = 0 # PhT triggers are only expected in triggered mode
chip_orders_ob = [[0, 1, 2, 3, 4, 5, 6], [8, 9, 10, 11, 12, 13, 14], [0, 2, 3, 4, 5, 6]]
"#;
    let custom_checks_file_name = "check_all_its_stave_custom_chip_id_order_errors.toml";
    let tmp_dir = assert_fs::TempDir::new()?;
    let tmp_custom_checks_path = tmp_dir.path().join(custom_checks_file_name);
    let mut custom_checks_file = std::fs::File::create(tmp_custom_checks_path.clone())?;
    std::io::Write::write_all(&mut custom_checks_file, custom_checks_str.as_bytes())?;

    custom_checks_file.sync_all()?;

    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg(FILE_CI_OLS_DATA_1HBF)
        .arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("--checks-toml")
        .arg(tmp_custom_checks_path);

    cmd.assert().success();
    match_on_out_no_case(&cmd.output()?.stderr, r"\[E9005\] chip id order", 0)?;
    // Trigger error is still present
    match_on_out_no_case(&cmd.output()?.stderr, "error.*expected.*PhT trigger", 1)?;

    Ok(())
}

#[test]
fn check_all_its_stave_custom_chip_id_order_errors_and_pht_adjusted_toml(
) -> Result<(), Box<dyn std::error::Error>> {
    // Check that errors are now gone when the faulty chip ordering
    //  due to a disabled ALPIDE chip is added to the valid ones in the TOML
    // Now the expected PhT trigger count is also correct
    let custom_checks_str = r#"
# Number of CRU Data Packets expected in the data
# Example value: 20 [type: u32]
# cdps = 10
# Number of Physics (PhT) Triggers expected in the data
# Example value: 20 [type: u32]
triggers_pht = 2 # PhT triggers are only expected in triggered mode
chip_orders_ob = [[0, 1, 2, 3, 4, 5, 6], [8, 9, 10, 11, 12, 13, 14], [0, 2, 3, 4, 5, 6]]
"#;
    let custom_checks_file_name = "check_all_its_stave_custom_chip_id_order_errors.toml";
    let tmp_dir = assert_fs::TempDir::new()?;
    let tmp_custom_checks_path = tmp_dir.path().join(custom_checks_file_name);
    let mut custom_checks_file = std::fs::File::create(tmp_custom_checks_path.clone())?;
    std::io::Write::write_all(&mut custom_checks_file, custom_checks_str.as_bytes())?;

    custom_checks_file.sync_all()?;

    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg(FILE_CI_OLS_DATA_1HBF)
        .arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("--checks-toml")
        .arg(tmp_custom_checks_path);

    cmd.assert().success();

    // Both errors accounted for with the new custom checks
    match_on_out_no_case(&cmd.output()?.stderr, r"\[E9005\] chip id order", 0)?;
    match_on_out_no_case(&cmd.output()?.stderr, "error.*expected.*PhT trigger", 0)?;

    Ok(())
}
