use predicates::str::{contains, is_match};

use crate::util::*;
mod util;

#[test]
fn view_rdh() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_DW_DDW0)
        .arg("view")
        .arg("rdh")
        .arg("-v2");

    cmd.assert()
        .success()
        .stdout(is_match(VIEW_RDH_REGEX_SANITY)?.count(2));

    Ok(())
}

#[test]
fn check_all_its_max_error_1() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;
    let expect_err_cnt = 1;

    cmd.arg(FILE_1_HBF_BAD_DW_DDW0)
        .arg("check")
        .arg("all")
        .arg("its")
        .args(["-v", "3"])
        .args(["-e", &expect_err_cnt.to_string()]);

    cmd.assert().success().stderr(contains("ERROR ").count(1));

    Ok(())
}

#[test]
fn check_all_its_max_error_2() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;
    let expect_err_cnt = 2;

    cmd.arg(FILE_1_HBF_BAD_DW_DDW0)
        .arg("check")
        .arg("all")
        .arg("its")
        .args(["-v", "0"])
        .args(["-e", &expect_err_cnt.to_string()]);

    cmd.assert()
        .success()
        .stderr(contains("ERROR ").count(expect_err_cnt));

    Ok(())
}

#[test]
fn check_all_its_max_error_3() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;
    let expect_err_cnt = 3;
    cmd.arg(FILE_1_HBF_BAD_DW_DDW0)
        .arg("check")
        .arg("all")
        .arg("its")
        .args(["-v", "5"])
        .args(["-e", &expect_err_cnt.to_string()]);

    cmd.assert()
        .success()
        .stderr(contains("ERROR ").count(expect_err_cnt));

    Ok(())
}

#[test]
fn check_all_its_max_error_4() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;
    let expect_err_cnt = 4;
    cmd.arg(FILE_1_HBF_BAD_DW_DDW0)
        .arg("check")
        .arg("all")
        .arg("its")
        .args(["-v", "3"])
        .args(["-e", &expect_err_cnt.to_string()]);

    cmd.assert()
        .success()
        .stderr(contains("ERROR ").count(expect_err_cnt));

    Ok(())
}

#[test]
fn check_all_its_max_error_5() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;
    let expect_err_cnt = 4; // There's only 4 errors in the data.
    cmd.arg(FILE_1_HBF_BAD_DW_DDW0)
        .arg("check")
        .arg("all")
        .arg("its")
        .args(["-v", "4"])
        .args(["-e", "5"]);

    cmd.assert()
        .success()
        .stderr(contains("ERROR ").count(expect_err_cnt));

    Ok(())
}

#[test]
fn view_its_readout_frame() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_1_HBF_BAD_DW_DDW0)
        .arg("view")
        .arg("its-readout-frames");
    use predicate::str::contains;
    // A Dataword and DDW0 has bad ID.
    cmd.assert().success().stderr(contains("ERROR ").count(2));
    cmd.assert().success().stdout(
        contains("RDH").count(2).and(
            contains("IHW").count(1).and(
                contains("TDH")
                    .count(1)
                    .and(contains("TDT").count(1).and(contains("DDW").count(0))),
            ),
        ),
    );

    Ok(())
}
