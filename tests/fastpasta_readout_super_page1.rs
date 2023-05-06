use crate::util::*;
mod util;

const REPORT_MATCH_PATTERNS: [&str; 6] = [
    "Trigger Type.*0x4813",
    "Trigger Type.*HB",
    "RDH.*Version.*7",
    "Total.*RDHs.*6",
    "Total.*hbfs.*3",
    "((layers)|(staves)).*((layers)|(staves)).*L1_6",
];

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
fn check_sanity() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_READOUT_SUPERPAGE_1).arg("check").arg("sanity");
    cmd.assert().success();

    for pattern in REPORT_MATCH_PATTERNS {
        match_on_out_no_case(&cmd.output()?.stdout, pattern, 1)?;
    }

    Ok(())
}

#[test]
fn check_all_its() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_READOUT_SUPERPAGE_1)
        .arg("check")
        .arg("all")
        .arg("its");
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;

    for pattern in REPORT_MATCH_PATTERNS {
        match_on_out_no_case(&cmd.output()?.stdout, pattern, 1)?;
    }

    Ok(())
}

#[test]
fn view_its_readout_frames() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_READOUT_SUPERPAGE_1)
        .arg("view")
        .arg("its-readout-frames");

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
fn check_all_its_stave() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg("check")
        .arg("all")
        .arg("its_stave")
        .arg("-v2")
        .arg("--filter-its-stave")
        .arg("L1_6")
        .arg(FILE_READOUT_SUPERPAGE_1);
    cmd.assert().success();

    assert_no_errors_or_warn(&cmd.output()?.stderr)?;

    match_on_out_no_case(&cmd.output()?.stdout, "its stave.*l1_6", 1)?;

    Ok(())
}
