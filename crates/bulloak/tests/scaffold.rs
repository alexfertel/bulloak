#![allow(missing_docs)]
use std::{env, fs};

use common::{cmd, get_binary_path};
use owo_colors::OwoColorize;
use pretty_assertions::assert_eq;

mod common;

/// Ensures behaviour is kept consistent across all the different backends, by running the same
/// assertion closure on the result of both. The closure's second parameter is filled with the
/// contents of the corresponding expected output file(if available), to account for the
/// differences in output of every backend
fn assert_on_all_parsers(
    treefile: &str,
    assertor: fn(std::process::Output, Option<String>),
) {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tests_path = cwd.join("tests").join("scaffold");
    let tree_path = tests_path.join(treefile.to_string());

    let expected_sol =
        fs::read_to_string(tree_path.with_extension("t.sol")).ok();
    let expected_noir =
        fs::read_to_string(tree_path.with_extension("t.nr")).ok();

    let solidity_output = cmd(&binary_path, "scaffold", &tree_path, &[]);
    assertor(solidity_output, expected_sol);
    let noir_output =
        cmd(&binary_path, "scaffold", &tree_path, &["-l", "noir"]);
    assertor(noir_output, expected_noir);
}

#[cfg(not(target_os = "windows"))]
#[test]
fn scaffolds_trees() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tests_path = cwd.join("tests").join("scaffold");
    let trees = [
        "basic.tree",
        "complex.tree",
        "multiple_roots.tree",
        "removes_invalid_title_chars.tree",
        "hash_pair.tree",
        "format_descriptions.tree",
        "revert_when.tree",
        "spurious_comments.tree",
    ];

    for tree_name in trees {
        let tree_path = tests_path.join(tree_name);
        let output = cmd(&binary_path, "scaffold", &tree_path, &[]);
        let actual = String::from_utf8(output.stdout).unwrap();

        let mut output_file = tree_path.clone();
        output_file.set_extension("t.sol");
        let expected = fs::read_to_string(output_file).unwrap();

        // We trim here because we don't care about ending newlines.
        assert_eq!(expected.trim(), actual.trim());
    }
}

#[cfg(not(target_os = "windows"))]
#[test]
fn scaffolds_trees_with_vm_skip() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tests_path = cwd.join("tests").join("scaffold");
    let trees = [
        "basic.tree",
        "complex.tree",
        "multiple_roots.tree",
        "removes_invalid_title_chars.tree",
    ];
    let args = vec!["--vm-skip"];

    for tree_name in trees {
        let tree_path = tests_path.join(tree_name);
        let output = cmd(&binary_path, "scaffold", &tree_path, &args);
        let actual = String::from_utf8(output.stdout).unwrap();

        let mut trimmed_extension = tree_path.clone();
        trimmed_extension.set_extension("");

        let mut output_file_str = trimmed_extension.into_os_string();
        output_file_str.push("_vm_skip");

        let mut output_file: std::path::PathBuf = output_file_str.into();
        output_file.set_extension("t.sol");

        let expected = fs::read_to_string(output_file).unwrap();

        // We trim here because we don't care about ending newlines.
        assert_eq!(expected.trim(), actual.trim());
    }
}

#[cfg(not(target_os = "windows"))]
#[test]
fn scaffolds_trees_with_format_descriptions() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tests_path = cwd.join("tests").join("scaffold");
    let trees = ["format_descriptions.tree"];
    let args = vec!["--format-descriptions"];

    for tree_name in trees {
        let tree_path = tests_path.join(tree_name);
        let output = cmd(&binary_path, "scaffold", &tree_path, &args);
        let actual = String::from_utf8(output.stdout).unwrap();

        let mut trimmed_extension = tree_path.clone();
        trimmed_extension.set_extension("");

        let mut output_file_str = trimmed_extension.into_os_string();
        output_file_str.push("_formatted");

        let mut output_file: std::path::PathBuf = output_file_str.into();
        output_file.set_extension("t.sol");

        let expected = fs::read_to_string(output_file).unwrap();

        // We trim here because we don't care about ending newlines.
        assert_eq!(expected.trim(), actual.trim());
    }
}

#[cfg(not(target_os = "windows"))]
#[test]
fn scaffolds_trees_with_skip_modifiers() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tests_path = cwd.join("tests").join("scaffold");
    let trees = ["skip_modifiers.tree"];

    for tree_name in trees {
        let tree_path = tests_path.join(tree_name);
        let output = cmd(&binary_path, "scaffold", &tree_path, &["-m"]);
        let actual = String::from_utf8(output.stdout).unwrap();

        let mut output_file = tree_path.clone();
        output_file.set_extension("t.sol");
        let expected = fs::read_to_string(output_file).unwrap();

        // We trim here because we don't care about ending newlines.
        assert_eq!(expected.trim(), actual.trim());
    }
}

#[cfg(not(target_os = "windows"))]
#[test]
fn skips_trees_when_file_exists() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tests_path = cwd.join("tests").join("scaffold");
    let trees = ["basic.tree", "complex.tree", "multiple_roots.tree"];

    for tree_name in trees {
        let tree_path = tests_path.join(tree_name);
        let output = cmd(&binary_path, "scaffold", &tree_path, &["-w"]);
        let actual = String::from_utf8(output.stderr).unwrap();

        let expected = format!("{}", "warn".yellow());
        assert!(actual.starts_with(&expected));
    }
}

#[cfg(not(target_os = "windows"))]
#[test]
fn errors_when_tree_is_empty() {
    assert_on_all_parsers("empty.tree", |output, _| {
        let actual = String::from_utf8(output.stderr).unwrap();
        assert_eq!(output.status.code().unwrap() , 1);
        assert!(String::from_utf8(output.stdout).unwrap().is_empty());
        assert!(actual.contains("found an empty tree"));
    });
}

#[test]
fn errors_when_condition_appears_multiple_times() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tests_path = cwd.join("tests").join("scaffold");
    let trees = ["duplicated_top_action.tree"];

    for tree_name in trees {
        let tree_path = tests_path.join(tree_name);
        let output = cmd(&binary_path, "scaffold", &tree_path, &[]);
        let actual = String::from_utf8(output.stderr).unwrap();

        assert!(actual.contains("found an identifier more than once"));
    }
}

#[cfg(not(target_os = "windows"))]
#[test]
fn errors_when_root_contract_identifier_is_missing_multiple_roots() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tests_path = cwd.join("tests").join("scaffold");
    let trees = ["contract_name_missing_multiple_roots.tree"];

    for tree_name in trees {
        let tree_path = tests_path.join(tree_name);
        let output = cmd(&binary_path, "scaffold", &tree_path, &[]);
        let actual = String::from_utf8(output.stderr).unwrap();

        assert!(actual.contains("contract name missing at tree root #1"));
    }
}

/// If you pass an invalid glob to `bulloak scaffold`,
/// it should warn but still exit code = 0 and produce no contract.
#[test]
fn scaffold_invalid_glob_warns_but_no_output() {
    let cwd = env::current_dir().unwrap();
    let bin = common::get_binary_path();

    // Deliberately invalid glob (unmatched '[').
    let bad_glob = cwd.join("tests").join("scaffold").join("*[.tree");
    let out = cmd(&bin, "scaffold", &bad_glob, &[]);

    assert!(
        out.status.success(),
        "scaffold should succeed even on invalid glob, got {:?}",
        out
    );

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("could not expand"),
        "did not see the expected warn: {}",
        stderr
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("contract "),
        "unexpected scaffold output: {}",
        stdout
    );
}

#[cfg(not(target_os = "windows"))]
#[test]
fn scaffold_dissambiguates_function_name_collisions() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tests_path = cwd.join("tests").join("scaffold");
    let trees = ["disambiguation.tree"];

    for tree_name in trees {
        let tree_path = tests_path.join(tree_name);
        let output = cmd(&binary_path, "scaffold", &tree_path, &[]);
        let actual = String::from_utf8(output.stdout).unwrap();

        let mut output_file = tree_path.clone();
        output_file.set_extension("t.sol");
        let expected = fs::read_to_string(output_file).unwrap();

        // We trim here because we don't care about ending newlines.
        assert_eq!(expected.trim(), actual.trim());
    }
}
