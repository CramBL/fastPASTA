use predicates::str::contains;

use crate::util::*;
mod util;

// Asserts that the end of processing report summary contains correct information
fn validate_report_summary(byte_output: &Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let match_patterns = vec![
        "(?i)Trigger Type.*0x6A03",
        "(?i)Trigger Type.*SOC",
        "(?i)RDH.*Version.*7",
        "(?i)Total.*RDHs.*10",
        "(?i)Total.*hbfs.*5",
        "(?i)((layers)|(staves)).*((layers)|(staves)).*L0_12",
    ];
    match_patterns.into_iter().for_each(|pattern| {
        assert!(match_on_output(byte_output, pattern, 1));
    });
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

    assert!(match_on_output(&cmd.output()?.stderr, "(?i)error - ", 0));
    assert!(match_on_output(&cmd.output()?.stderr, "(?i)warn - ", 0));

    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

#[test]
fn check_sanity_its() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_10_RDH).arg("check").arg("sanity").arg("its");
    cmd.assert().success();

    assert!(match_on_output(&cmd.output()?.stderr, "(?i)error - ", 0));
    assert!(match_on_output(&cmd.output()?.stderr, "(?i)warn - ", 0));
    // Asserts on stdout
    validate_report_summary(&cmd.output()?.stdout)?;

    Ok(())
}

#[test]
fn check_all() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_10_RDH).arg("check").arg("all");
    cmd.assert().success();

    assert!(match_on_output(&cmd.output()?.stderr, "(?i)error - ", 0));
    assert!(match_on_output(&cmd.output()?.stderr, "(?i)warn - ", 0));
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
        .arg("its_stave")
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
        .arg("its_stave")
        .arg("-v2")
        .arg("--its-trigger-period")
        .arg("1")
        .arg("--filter-its-stave")
        .arg("L3_2");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;

    assert!(match_on_output(
        &cmd.output()?.stdout,
        "(?i)its stave.*<<none>>.*not found.*l3_2",
        1
    ));

    Ok(())
}
#[test]
fn check_all_its_trigger_period_mismatch() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_10_RDH)
        .arg("check")
        .arg("all")
        .arg("its_stave")
        .arg("-v3")
        .arg("--its-trigger-period")
        .arg("1")
        .arg("--filter-its-stave")
        .arg("L0_12");
    cmd.assert().success();

    assert!(match_on_output(&cmd.output()?.stderr, "(?i)error - ", 4));
    assert!(match_on_output(&cmd.output()?.stderr, "(?i)warn - ", 0));

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
        .arg("its_stave")
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
fn check_all_its_stave() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_10_RDH)
        .arg("check")
        .arg("all")
        .arg("its_stave")
        .arg("-v4")
        .arg("--filter-its-stave")
        .arg("L0_12");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;

    match_on_out_no_case(&cmd.output()?.stdout, "its stave.*l0_12", 1)?;

    Ok(())
}

#[test]
fn check_all_its_stave_missing_filter() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_10_RDH)
        .arg("check")
        .arg("all")
        .arg("its_stave")
        .arg("-v4");
    cmd.assert().failure();

    match_on_out_no_case(&cmd.output()?.stderr, "invalid.*specify.*stave", 1)?;

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

    assert!(match_on_output(&cmd.output()?.stdout, r"(?i).*L0_12", 1));

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

    assert!(match_on_output(&cmd.output()?.stderr, "(?i)error - ", 0));
    assert!(match_on_output(&cmd.output()?.stderr, "(?i)warn - ", 0));
    // Asserts on stdout
    assert!(match_on_output(
        &cmd.output()?.stdout,
        "(?i)Total.*RDHs.*10",
        1
    ));
    // Checking the filtered stats
    assert!(match_on_output(
        &cmd.output()?.stdout,
        r"(?i).*filter.*stats",
        1
    ));
    assert!(match_on_output(
        &cmd.output()?.stdout,
        &(r"(?i)FEE.*".to_string() + fee_id_to_filter),
        1
    ));

    assert!(match_on_output(
        &cmd.output()?.stdout,
        r"(?i)\|.* RDHs.*10",
        1
    ));

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

    assert!(match_on_output(&cmd.output()?.stderr, "(?i)error - ", 0));
    assert!(match_on_output(&cmd.output()?.stderr, "(?i)warn - ", 0));
    // Asserts on stdout
    assert!(match_on_output(
        &cmd.output()?.stdout,
        "(?i)Total.*RDHs.*10",
        1
    ));
    // Checking the filtered stats
    assert!(match_on_output(
        &cmd.output()?.stdout,
        r"(?i).*filter.*stats",
        1
    ));
    assert!(match_on_output(
        &cmd.output()?.stdout,
        &(r"(?i)FEE.*not found.*".to_string() + fee_id_to_filter),
        1
    ));

    assert!(match_on_output(
        &cmd.output()?.stdout,
        r"(?i)\|.* RDHs.* 0 ",
        1
    ));

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
