use assert_cmd::prelude::*; // Add methods on commands
use predicate::str::is_match;
use predicates::prelude::*; // Used for writing assertions
use std::process::Command;
mod fastpasta;
use crate::fastpasta::match_on_output;
use crate::fastpasta::FILE_1_HBF_BAD_CDP_STRUCTURE; // File used in these tests

#[test]
fn view_rdh() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_CDP_STRUCTURE)
        .arg("view")
        .arg("rdh")
        .arg("-v2");

    cmd.assert()
        .success()
        .stdout(is_match(": .* (7|6) .* 64 .* (0|2)")?.count(2));

    Ok(())
}

#[test]
fn check_sanity_its() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_CDP_STRUCTURE)
        .arg("check")
        .arg("sanity")
        .arg("its")
        .arg("-v3");

    // All data is correct individually, so a sanity check should pass
    cmd.assert().stderr(is_match("ERROR -")?.count(0));
    cmd.assert().stderr(is_match("WARN -")?.count(0));

    Ok(())
}

#[test]
fn check_all_its() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_CDP_STRUCTURE)
        .arg("check")
        .arg("all")
        .arg("its")
        .arg("-v4");

    // 1 Error from a stateful check
    cmd.assert().stderr(is_match("ERROR -")?.count(1));
    cmd.assert().stderr(is_match("WARN -")?.count(0));

    Ok(())
}
