#![allow(missing_docs)]
use std::{env, path::PathBuf};

use common::cmd;

mod common;

fn unmatched_recursive_glob(cwd: &PathBuf) -> PathBuf {
    cwd.join("tests").join("does-not-exist").join("**").join("*.tree")
}

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

#[test]
fn check_succeeds_when_recursive_glob_matches_no_files() {
    let cwd = env::current_dir().unwrap();
    let bin = common::get_binary_path();

    let glob_pattern = unmatched_recursive_glob(&cwd);
    let out = cmd(&bin, "check", &glob_pattern, &[]);
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("All checks completed successfully"));
    assert!(out.stderr.is_empty());
}

#[test]
fn scaffold_succeeds_when_recursive_glob_matches_no_files() {
    let cwd = env::current_dir().unwrap();
    let bin = common::get_binary_path();

    let glob_pattern = unmatched_recursive_glob(&cwd);
    let out = cmd(&bin, "scaffold", &glob_pattern, &[]);
    assert!(out.status.success());
    assert!(out.stdout.is_empty());
    assert!(out.stderr.is_empty());
}
