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
