use crate::util::*;
mod util;

#[test]
fn bad_file_input_check_sanity_its() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg("../LICENSE")
        .arg("check")
        .arg("sanity")
        .arg("its")
        .arg("-v4");
    cmd.assert().failure();

    // Check that an error is printed to stderr showing the file is not a valid input
    match_on_out_no_case(&cmd.output()?.stderr, "processing failed", 1)?;
    // No mention of RDH in stdout as the file is not a valid input
    match_on_out_no_case(&cmd.output()?.stdout, "rdh", 0)?;

    Ok(())
}

#[test]
fn bad_file_input_view_its_readout_frames() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg("../LICENSE")
        .arg("view")
        .arg("its-readout-frames")
        .arg("-v4");
    cmd.assert().failure();

    // Check that an error is printed to stderr showing the file is not a valid input
    match_on_out_no_case(&cmd.output()?.stderr, "processing failed", 1)?;
    // No mention of RDH in stdout as the file is not a valid input
    match_on_out_no_case(&cmd.output()?.stdout, "rdh", 0)?;

    Ok(())
}

#[test]
fn bad_filter_link_value() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;
    // No file arg is needed when other args are invalid
    cmd.arg("check")
        .arg("sanity")
        .arg("-v4")
        .arg("--filter-link")
        .arg("256"); // 256 is not a valid link value, max is 255
    cmd.assert().failure();

    // Check that an error is printed to stderr showing the file is not a valid input
    match_on_out_no_case(&cmd.output()?.stderr, "invalid value", 1)?;
    // No mention of RDH in stdout as the file is not a valid input
    match_on_out_no_case(&cmd.output()?.stdout, "rdh", 0)?;

    Ok(())
}

/// Check that a not found file returns a fatal error, with a description of an OS error
///
/// Try with all the different verbosity values 0-4
#[test]
fn file_doesnt_exist_v0() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;

    cmd.arg("test/file/doesnt/exist")
        .arg("check")
        .arg("sanity")
        .arg("-v0");
    cmd.assert().failure().stderr(
        predicate::str::contains("ERROR - FATAL:").and(predicate::str::contains("os error")),
    );
    Ok(())
}
#[test]
fn file_doesnt_exist_v1() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg("test/file/doesnt/exist")
        .arg("check")
        .arg("sanity")
        .arg("-v1");

    cmd.assert().failure().stderr(
        predicate::str::contains("ERROR - FATAL:").and(predicate::str::contains("os error")),
    );

    Ok(())
}

#[test]
fn file_doesnt_exist_v2() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg("test/file/doesnt/exist")
        .arg("check")
        .arg("sanity")
        .arg("-v2");

    cmd.assert().failure().stderr(
        predicate::str::contains("ERROR - FATAL:").and(predicate::str::contains("os error")),
    );

    Ok(())
}

#[test]
fn file_doesnt_exist_v3() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg("test/file/doesnt/exist")
        .arg("check")
        .arg("sanity")
        .arg("-v3");

    cmd.assert().failure().stderr(
        predicate::str::contains("ERROR - FATAL:").and(predicate::str::contains("os error")),
    );
    Ok(())
}

#[test]
fn file_doesnt_exist_v4() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fastpasta")?;
    cmd.arg("test/file/doesnt/exist")
        .arg("check")
        .arg("sanity")
        .arg("-v4");

    cmd.assert().failure().stderr(
        predicate::str::contains("ERROR - FATAL:").and(predicate::str::contains("os error")),
    );

    Ok(())
}
