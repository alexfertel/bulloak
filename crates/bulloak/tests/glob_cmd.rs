#![allow(missing_docs)]
use std::{env, process::Command};

mod common;

#[test]
fn scaffold_expands_glob_internally() {
    let cwd = env::current_dir().unwrap();
    let bin = common::get_binary_path();

    // Build the pattern via PathBuf so it uses "\" on Windows.
    let glob_pattern = cwd
        .join("tests")
        .join("scaffold")
        .join("*.tree")
        .to_string_lossy()
        .to_string();

    let out = Command::new(&bin)
        .arg("scaffold")
        .arg(&glob_pattern)
        .output()
        .expect("should run scaffold");

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("contract HashPair"),);
    assert!(stdout.contains("contract CancelTest"),);
}

#[cfg(not(target_os = "windows"))]
#[test]
fn check_expands_glob_internally() {
    let cwd = env::current_dir().unwrap();
    let bin = common::get_binary_path();

    let glob_pattern = cwd
        .join("tests")
        .join("check")
        .join("*.tree")
        .to_string_lossy()
        .to_string();

    let out = Command::new(&bin)
        .arg("check")
        .arg(&glob_pattern)
        .arg("-m")
        .output()
        .expect("should run check");
    assert!(!out.status.success());

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("contract name mismatch"));
    assert!(stderr.contains("contract name missing at tree root"));
}
