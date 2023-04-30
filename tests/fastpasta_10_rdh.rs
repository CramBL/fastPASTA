use assert_cmd::prelude::*; // Add methods on commands
use predicate::str::is_match;
use predicates::prelude::*; // Used for writing assertions
use std::process::Command;
mod fastpasta;
use crate::fastpasta::match_on_output;
use crate::fastpasta::FILE_10_RDH; // File used in these tests

#[test]
fn view_rdh() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_10_RDH).arg("view").arg("rdh").arg("-v2");

    cmd.assert()
        .success()
        .stdout(is_match(": .* (7|6) .* 64 .* (0|2)").unwrap().count(10));

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

    assert!(match_on_output(
        &cmd.output().unwrap().stderr,
        "(?i)error - ",
        0
    ));
    assert!(match_on_output(
        &cmd.output().unwrap().stderr,
        "(?i)warn - ",
        0
    ));
    // Asserts on stdout
    let stdout = &cmd.output().unwrap().stdout;
    let match_patterns = vec![
        "(?i)Trigger Type.*0x6A03",
        "(?i)Trigger Type.*SOC",
        "(?i)RDH.*Version.*7",
        "(?i)Total.*RDHs.*10",
        "(?i)Total.*hbfs.*5",
        "(?i)((layers)|(staves)).*((layers)|(staves)).*L0_12",
    ];
    let expected_matches = 1;

    match_patterns.into_iter().for_each(|pattern| {
        assert!(match_on_output(stdout, pattern, expected_matches));
    });

    Ok(())
}
