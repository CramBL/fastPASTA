use assert_cmd::prelude::*; // Add methods on commands
use predicate::str::is_match;
use predicates::prelude::*; // Used for writing assertions
use std::process::Command;
mod fastpasta;
use crate::fastpasta::{match_on_output, FILE_READOUT_SUPERPAGE_1};

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
