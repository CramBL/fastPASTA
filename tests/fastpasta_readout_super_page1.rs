use crate::util::*;
mod util;

const REPORT_MATCH_PATTERNS: [&str; 6] = [
    "(?i)Trigger Type.*0x4813",
    "(?i)Trigger Type.*HB",
    "(?i)RDH.*Version.*7",
    "(?i)Total.*RDHs.*6",
    "(?i)Total.*hbfs.*3",
    "(?i)((layers)|(staves)).*((layers)|(staves)).*L1_6",
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
        assert!(match_on_output(&cmd.output()?.stdout, pattern, 1));
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

    cmd.assert().stderr(is_match("ERROR -")?.count(0));
    cmd.assert().stderr(is_match("WARN -")?.count(0));

    for pattern in REPORT_MATCH_PATTERNS {
        assert!(match_on_output(&cmd.output()?.stdout, pattern, 1));
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
