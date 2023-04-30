use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

/// Path to test files : tests/regression/test-data/
/// Files
const FILE_10_RDH: &str = "tests/regression/test-data/10_rdh.raw";
const FILE_ERR_NOT_HBF: &str = "tests/regression/test-data/err_not_hbf.raw";
const FILE_THRS_CDW_LINKS: &str = "tests/regression/test-data/thrs_cdw_links.raw";
const FILE_READOUT_SUPERPAGE_1: &str = "tests/regression/test-data/readout.superpage.1.raw";

#[test]
fn file_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg("test/file/doesnt/exist").arg("check").arg("sanity");
    cmd.assert().failure().stderr(
        predicate::str::contains("ERROR - FATAL:").and(predicate::str::contains("os error")),
    );

    Ok(())
}

#[test]
fn file_exists_exit_successful() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_10_RDH).arg("check").arg("sanity").arg("-v2");
    cmd.assert().success();

    Ok(())
}

#[test]
fn view_rdh() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_10_RDH).arg("view").arg("rdh").arg("-v2");

    use predicate::str::is_match;
    cmd.assert()
        .success()
        .stdout(is_match(": .* (7|6) .* 64 .* (0|2)").unwrap().count(10));

    Ok(())
}

#[test]
fn view_hbf() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_READOUT_SUPERPAGE_1).arg("view").arg("hbf");

    cmd.assert().success().stdout(
        predicate::str::contains("IHW").count(3).and(
            predicate::str::contains("TDH").count(3).and(
                predicate::str::contains("TDT")
                    .count(3)
                    .and(predicate::str::contains("DDW").count(3)),
            ),
        ),
    );

    Ok(())
}

#[test]
fn err_not_hbf_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;
    let re = fancy_regex::Regex::new(r"(?i)ERrOR - 0xa0.*pages").unwrap(); // case insensitive
    let pred_one_match = predicate::function(|&x| re.find_iter(x).count() == 1);

    let test_is_false = pred_one_match.eval(&"test");
    assert!(!test_is_false);
    let test_is_true = pred_one_match.eval(&"error - 0xa0 something pages something");
    assert!(test_is_true);

    cmd.arg(FILE_ERR_NOT_HBF).arg("check").arg("all");

    cmd.assert().success();

    // Take the output of stderr and convert it to string
    let res = cmd.output().unwrap().stderr;
    let str_res = std::str::from_utf8(&res).expect("invalid utf-8 sequence");

    // Compare with regex predicate
    assert!(pred_one_match.eval(&str_res));

    Ok(())
}
