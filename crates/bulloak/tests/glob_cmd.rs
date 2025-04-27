#![allow(missing_docs)]
use std::env;

use common::cmd;

mod common;

#[test]
fn scaffold_expands_glob_internally() {
    let cwd = env::current_dir().unwrap();
    let bin = common::get_binary_path();

    // Build the pattern via PathBuf so it uses "\" on Windows.
    let glob_pattern = cwd.join("tests").join("scaffold").join("*.tree");
    let out = cmd(&bin, "scaffold", &glob_pattern, &[]);
    assert!(!out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("contract HashPair"),);
    assert!(stdout.contains("contract CancelTest"),);
}

#[cfg(not(target_os = "windows"))]
#[test]
fn check_expands_glob_internally() {
    let cwd = env::current_dir().unwrap();
    let bin = common::get_binary_path();

    let glob_pattern = cwd.join("tests").join("check").join("*.tree");
    let out = cmd(&bin, "check", &glob_pattern, &[]);
    assert!(!out.status.success());

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("contract name mismatch"));
    assert!(stderr.contains("contract name missing at tree root"));
}
