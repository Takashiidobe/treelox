use std::process::Output;

use assert_cmd::cargo::CommandCargoExt;
use insta::{assert_yaml_snapshot, glob};
use insta_cmd::Command;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct TestOutput {
    status: i32,
    stdout: Vec<String>,
    stderr: Vec<String>,
}

#[test]
fn reference_files() {
    glob!("../test-files", "**/*.lox", |path| {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

        let Output {
            status,
            stdout,
            stderr,
        } = cmd.arg(path).output().unwrap();

        let test_output = TestOutput {
            status: status.code().unwrap(),
            stdout: String::from_utf8_lossy(&stdout)
                .to_string()
                .lines()
                .map(|x| x.to_owned())
                .collect(),
            stderr: String::from_utf8_lossy(&stderr)
                .to_string()
                .lines()
                .map(|x| x.to_owned())
                .collect(),
        };

        assert_yaml_snapshot!(test_output);
    });
}
