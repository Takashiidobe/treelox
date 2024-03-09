use assert_cmd::{assert::OutputAssertExt, cargo::CommandCargoExt};
use insta::{assert_debug_snapshot, glob};
use insta_cmd::Command;

#[test]
fn test_lox() {
    glob!("../test-files", "*.lox", |path| {
        let mut cmd = Command::cargo_bin("treelox").unwrap();

        assert_debug_snapshot!(cmd.arg(path).assert());
    });
}
