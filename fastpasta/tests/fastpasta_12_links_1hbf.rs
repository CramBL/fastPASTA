use crate::util::*;
use predicate::str::contains;
mod util;

const RDH_COUNT: usize = 39;
const IHW_COUNT: usize = 27;
const TDH_TDT_COUNT: usize = 59;
const DDW_COUNT: usize = 12;

#[test]
fn check_all_its_stave() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg(FILE_12_LINKS_1HBF)
        .args(["check", "all", "its-stave"]);
    cmd.assert().success();

    let (stdout, stderr) = (cmd.output()?.stdout, cmd.output()?.stderr);

    match_on_out_no_case(&stdout, "errors.*0", 1)?;
    match_on_out_no_case(&stdout, "total hbfs.*12", 1)?;
    match_on_out_no_case(&stderr, "error", 0)?;

    Ok(())
}

#[test]
fn view_its_readout_frames() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg("view")
        .arg("its-readout-frames")
        .arg(FILE_12_LINKS_1HBF);

    cmd.assert().success().stdout(
        contains("RDH").count(RDH_COUNT).and(
            contains("IHW").count(IHW_COUNT).and(
                contains("TDH").count(TDH_TDT_COUNT).and(
                    contains("TDT")
                        .count(TDH_TDT_COUNT)
                        .and(contains("DDW").count(DDW_COUNT)),
                ),
            ),
        ),
    );

    Ok(())
}
