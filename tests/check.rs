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
    let actual = String::from_utf8(output.stderr).unwrap();

    let expected = r#"Codegen not found: Couldn't find a corresponding element for "test_GivenTheSenderDoesNotImplementTheHook" in the solidity file.
Invalid codegen order: Found a matching element for "givenTheSenderImplementsTheHook", but the order is not correct.
Invalid codegen order: Found a matching element for "givenTheStreamsStatusIsCANCELED", but the order is not correct.
Invalid codegen order: Found a matching element for "givenTheStreamsStatusIsSETTLED", but the order is not correct.
Invalid codegen order: Found a matching element for "test_RevertGiven_TheStreamsStatusIsCANCELED", but the order is not correct.
Invalid codegen order: Found a matching element for "test_RevertGiven_TheStreamsStatusIsDEPLETED", but the order is not correct.
Invalid codegen order: Found a matching element for "test_RevertGiven_TheStreamsStatusIsSETTLED", but the order is not correct.
Invalid codegen order: Found a matching element for "whenTheSenderReverts", but the order is not correct."#;

    // We trim here because we don't care about ending newlines.
    assert_eq!(expected.trim(), actual.trim());
}

#[test]
fn checks_valid_structural_match() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tree_path = cwd.join("tests").join("check").join("extra_codegen.tree");

    let output = cmd(&binary_path, "check", &tree_path, &[]);
    let actual = String::from_utf8(output.stderr).unwrap();

    // We trim here because we don't care about ending newlines.
    assert_eq!("", actual);
}

#[test]
fn checks_missing_sol_file() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tree_path = cwd.join("tests").join("check").join("no_matching_sol.tree");

    let output = cmd(&binary_path, "check", &tree_path, &[]);
    let actual = String::from_utf8(output.stderr).unwrap();

    // We trim here because we don't care about ending newlines.
    assert!(actual.contains("File not found"));
    assert!(actual.contains("no_matching_sol.tree"));
}

#[test]
fn checks_empty_contract() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tree_path = cwd.join("tests").join("check").join("empty_contract.tree");

    let output = cmd(&binary_path, "check", &tree_path, &[]);
    let actual = String::from_utf8(output.stderr).unwrap();

    println!("{actual}");
    // We trim here because we don't care about ending newlines.
    assert!(actual.contains("Codegen not found"));
    assert!(actual.contains("test_ShouldNeverRevert"));
    assert!(actual.contains("test_ShouldNotFindTheSolidityFile"));
}
