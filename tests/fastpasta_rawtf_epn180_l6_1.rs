use crate::util::*;
mod util;

const LANE_WITH_ERRORS: u8 = 2;

// Asserts that the end of processing report summary contains correct information
fn validate_report_summary(byte_output: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let match_patterns = vec![
        "Trigger Type.*0x6803",
        "RDH.*Version.*7",
        "Total.*RDHs.*3",
        "Total.*hbfs.*1",
        "((layers)|(staves)).*((layers)|(staves)).*L6_1",
    ];
    for pattern in match_patterns {
        match_on_out_no_case(byte_output, pattern, 1)?;
    }

    Ok(())
}

#[test]
fn check_sanity() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_RAWTF_EPN180_L6_1).arg("check").arg("sanity");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;

    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

#[test]
fn check_all_its_stave_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_RAWTF_EPN180_L6_1)
        .arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("--filter-its-stave")
        .arg("l0_0");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;

    match_on_out_no_case(&cmd.output()?.stdout, "errors.*0", 1)?;
    match_on_out_no_case(&cmd.output()?.stdout, ".*not found.*l0_0", 1)?;

    Ok(())
}

#[test]
fn check_all_its_stave_filter() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    const EXPECTED_ALPIDE_ERRORS: u8 = 18;

    cmd.arg(FILE_RAWTF_EPN180_L6_1)
        .arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("--filter-its-stave")
        .arg("L6_1");
    cmd.assert().success();

    match_on_out_no_case(
        &cmd.output()?.stdout,
        &format!("errors.*{EXPECTED_ALPIDE_ERRORS}"),
        1,
    )?;

    match_on_out_no_case(
        &cmd.output()?.stderr,
        // Errors that have ALPIDE and lane 66 in them
        &format!("error - 0x.*alpide.*lane.*{LANE_WITH_ERRORS}"),
        EXPECTED_ALPIDE_ERRORS.into(),
    )?;
    match_on_out_no_case(&cmd.output()?.stdout, ".*stave.*l6_1", 1)?;

    Ok(())
}

#[test]
fn check_all_its_stave_trigger_period() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    const EXPECTED_ALPIDE_ERRORS: u8 = 18;

    cmd.arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("--filter-its-stave")
        .arg("L6_1")
        .arg(FILE_RAWTF_EPN180_L6_1)
        .arg("--its-trigger-period")
        .arg("198")
        .arg("-v4");
    cmd.assert().success();

    match_on_out_no_case(
        &cmd.output()?.stderr,
        "error - 0x",
        EXPECTED_ALPIDE_ERRORS.into(),
    )?;
    match_on_out_no_case(&cmd.output()?.stdout, ".*stave.*l6_1", 1)?;

    Ok(())
}

#[test]
fn check_all_its_stave_bad_trigger_period() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    const EXPECTED_ALPIDE_ERRORS: u8 = 18;
    const EXPECTED_TRIGGER_PERIOD_ERRORS: u8 = 18 - 1; // There's 18 TDTs, but a period is from one TDT to the next so there's N-1 periods.

    cmd.arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("-s")
        .arg("l6_1")
        .arg(FILE_RAWTF_EPN180_L6_1)
        .arg("-p")
        .arg("1337")
        .arg("-v4");
    cmd.assert().success();

    match_on_out_no_case(
        &cmd.output()?.stderr,
        "error - 0x",
        (EXPECTED_ALPIDE_ERRORS + EXPECTED_TRIGGER_PERIOD_ERRORS).into(),
    )?;
    match_on_out_no_case(&cmd.output()?.stdout, ".*stave.*l6_1", 1)?;

    Ok(())
}

#[test]
fn view_hbf() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg("view")
        .arg("its-readout-frames")
        .arg(FILE_RAWTF_EPN180_L6_1);

    use predicate::str::contains;
    cmd.assert().success().stdout(
        contains("RDH").count(3).and(
            contains("IHW").count(2).and(
                contains("TDH")
                    .count(25)
                    .and(contains("TDT").count(18).and(contains("DDW").count(1))),
            ),
        ),
    );

    Ok(())
}

#[test]
fn check_all_its_stave() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    const EXPECTED_ALPIDE_ERRORS: u8 = 18;

    cmd.arg(FILE_RAWTF_EPN180_L6_1)
        .arg("check")
        .arg("all")
        .arg("its-stave");
    cmd.assert().success();

    match_on_out_no_case(
        &cmd.output()?.stdout,
        &format!("errors.*{EXPECTED_ALPIDE_ERRORS}"),
        1,
    )?;

    match_on_out_no_case(
        &cmd.output()?.stderr,
        // Errors that have ALPIDE and lane 66 in them
        &format!("error - 0x.*alpide.*lane.*{LANE_WITH_ERRORS}"),
        EXPECTED_ALPIDE_ERRORS.into(),
    )?;
    match_on_out_no_case(&cmd.output()?.stdout, ".*stave.*l6_1", 1)?;

    Ok(())
}

#[test]
fn check_all_its_stave_mute_errors() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    const EXPECTED_ALPIDE_ERRORS: u8 = 18;

    cmd.arg(FILE_RAWTF_EPN180_L6_1)
        .arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("--mute-errors");
    cmd.assert().success();

    match_on_out_no_case(
        &cmd.output()?.stdout,
        &format!("errors.*{EXPECTED_ALPIDE_ERRORS}"),
        1,
    )?;

    match_on_out_no_case(
        &cmd.output()?.stderr,
        // Errors that have ALPIDE and lane 66 in them
        "error - 0x.*alpide.*lane.*66",
        // Expect 0 error messages as they are muted
        0,
    )?;
    match_on_out_no_case(&cmd.output()?.stdout, ".*stave.*l6_1", 1)?;

    Ok(())
}

#[test]
fn check_all_its_stave_no_chip_order_errors() -> Result<(), Box<dyn std::error::Error>> {
    // Checks that the chip order errors are not present now that they are optional (custom check)
    let mut cmd = Command::cargo_bin("fastpasta")?;

    const EXPECTED_ALPIDE_ERRORS: u8 = 18; // Bunch counter mismatch errors

    cmd.arg(FILE_RAWTF_EPN180_L6_1)
        .arg("check")
        .arg("all")
        .arg("its-stave");
    cmd.assert().success();

    match_on_out_no_case(
        &cmd.output()?.stdout,
        &format!("errors.*{EXPECTED_ALPIDE_ERRORS}"),
        1,
    )?;

    match_on_out_no_case(&cmd.output()?.stderr, "chip id order", 0)?;

    Ok(())
}

#[test]
fn check_all_its_stave_custom_chip_id_order_errors() -> Result<(), Box<dyn std::error::Error>> {
    // Check that errors are present when a custom check on the chip id order is used
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

    const EXPECTED_ALPIDE_ERRORS: u8 = 18;

    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg(FILE_RAWTF_EPN180_L6_1)
        .arg("check")
        .arg("all")
        .arg("its-stave")
        .arg("--checks-toml")
        .arg(tmp_custom_checks_path);

    cmd.assert().success();

    match_on_out_no_case(
        &cmd.output()?.stdout,
        &format!("errors.*{EXPECTED_ALPIDE_ERRORS}"),
        1,
    )?;

    match_on_out_no_case(&cmd.output()?.stderr, "chip id order", 18)?;

    Ok(())
}
