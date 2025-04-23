use assert_cmd::Command;

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("sam-cli").unwrap();
    cmd.arg("--help");
    cmd.assert().success();
}