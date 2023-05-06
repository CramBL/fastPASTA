use crate::util::*;
mod util;

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

#[test]
fn check_all_its_err_msg() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_CDP_STRUCTURE)
        .arg("check")
        .arg("all")
        .arg("its")
        .arg("-v2");

    // 1 Error from a stateful check
    // Eror message should indicate: In position 0xE0, something about DDW0 and RDH
    match_on_out_no_case(&cmd.output()?.stderr, "0xe0.*(DDW0|RDH).*(DDW0|RDH)", 1)?;

    Ok(())
}
