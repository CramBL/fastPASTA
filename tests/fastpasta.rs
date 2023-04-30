use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

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

    cmd.arg("tests/regression/test-data/10_rdh.raw")
        .arg("check")
        .arg("sanity")
        .arg("-v2");
    cmd.assert().success();

    Ok(())
}

#[test]
fn view_rdh() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg("tests/regression/test-data/10_rdh.raw")
        .arg("view")
        .arg("rdh")
        .arg("-v2");

    use predicate::str::is_match;
    cmd.assert()
        .success()
        .stdout(is_match(": .* (7|6) .* 64 .* (0|2)").unwrap().count(10));

    Ok(())
}

#[test]
fn view_hbf() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg("tests/regression/test-data/readout.superpage.1.raw")
        .arg("view")
        .arg("hbf");

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
