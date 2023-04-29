use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

#[test]
fn file_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg("test/file/doesnt/exist").arg("check").arg("sanity");
    cmd.assert().failure().stderr(predicate::str::contains(
        "ERROR - FATAL: The system cannot find the path",
    ));

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
    cmd.assert().success();

    Ok(())
}
