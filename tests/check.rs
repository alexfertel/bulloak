use std::env;

use pretty_assertions::assert_eq;

use common::cmd;
use common::get_binary_path;

mod common;

#[test]
fn checks_invalid_structural_match() {
    let binary_path = get_binary_path();
    let cwd = env::current_dir().unwrap();
    let tree_path = cwd
        .join("tests")
        .join("check")
        .join("invalid_sol_structure.tree");

    let output = cmd(&binary_path, "check", &tree_path, &[]);
    let stderr = String::from_utf8(output.stderr).unwrap();
    let actual = stderr
        .lines()
        .filter(|line| line.starts_with("check failed:"));

    let expected = r#"check failed: couldn't find a corresponding element for "givenTheStreamIsCold" in the Solidity file
check failed: couldn't find a corresponding element for "whenTheSenderDoesNotRevert" in the Solidity file
check failed: found a matching element for "test_RevertGiven_TheStreamsStatusIsCANCELED" in line 11, but the order is not correct
check failed: found a matching element for "test_RevertGiven_TheStreamsStatusIsSETTLED" in line 13, but the order is not correct
check failed: found a matching element for "test_WhenTheSenderReverts" in line 29, but the order is not correct
check failed: found a matching element for "test_WhenThereIsReentrancy" in line 35, but the order is not correct"#.lines();

    for (expected, actual) in expected.zip(actual) {
        assert_eq!(expected, actual);
    }
}

#[test]
fn checks_valid_structural_match() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tree_path = cwd.join("tests").join("check").join("extra_codegen.tree");

    let output = cmd(&binary_path, "check", &tree_path, &[]);
    let stderr = String::from_utf8(output.stderr).unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!("", stderr);
    assert!(stdout.contains("All checks completed successfully! No issues found."));
}

#[test]
fn checks_missing_sol_file() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tree_path = cwd.join("tests").join("check").join("no_matching_sol.tree");

    let output = cmd(&binary_path, "check", &tree_path, &[]);
    let actual = String::from_utf8(output.stderr).unwrap();

    assert!(actual.contains("check failed: the file is missing its matching Solidity file"));
    assert!(actual.contains("no_matching_sol.tree"));
}

#[test]
fn checks_empty_contract() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tree_path = cwd.join("tests").join("check").join("empty_contract.tree");

    let output = cmd(&binary_path, "check", &tree_path, &[]);
    let actual = String::from_utf8(output.stderr).unwrap();

    assert!(actual.contains(r#"check failed: couldn't find a corresponding element for "test_ShouldNeverRevert" in the Solidity file"#));
    assert!(actual.contains(r#"check failed: couldn't find a corresponding element for "test_ShouldNotFindTheSolidityFile" in the Solidity file"#));
}

#[test]
fn checks_missing_contract() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tree_path = cwd
        .join("tests")
        .join("check")
        .join("missing_contract.tree");

    let output = cmd(&binary_path, "check", &tree_path, &[]);
    let actual = String::from_utf8(output.stderr).unwrap();

    assert!(actual.contains(r#"check failed: couldn't find a corresponding contract for "MissingContract" in the Solidity file"#));
}

#[test]
fn checks_contract_name_mismatch() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tree_path = cwd
        .join("tests")
        .join("check")
        .join("contract_names_mismatch.tree");

    let output = cmd(&binary_path, "check", &tree_path, &[]);
    let actual = String::from_utf8(output.stderr).unwrap();

    assert!(actual.contains(r#"check failed: couldn't find a corresponding contract for "ADifferentName" in the Solidity file. Found "ContractName""#));
}
