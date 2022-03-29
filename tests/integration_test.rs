use anyhow::Result;
use assert_cmd::Command;

#[test]
fn run_ok_script() -> Result<()> {
    let mut cmd = Command::cargo_bin("shunit")?;
    cmd.args(&["./test/im_ok.sh"]).assert().success();
    Ok(())
}

#[test]
fn run_fail_script() -> Result<()> {
    let mut cmd = Command::cargo_bin("shunit")?;
    cmd.args(&["./test/bad_apple.sh"]).assert().success();
    Ok(())
}

#[test]
fn run_a_suite() -> Result<()> {
    let mut cmd = Command::cargo_bin("shunit")?;
    cmd.arg("./test/bad_apple.sh");
    cmd.arg("./test/im_ok.sh");
    cmd.arg("./test/JUnit.xml");
    cmd.arg("./test/slow.sh");
    cmd.assert().success();
    Ok(())
}
