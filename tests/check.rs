use std::{
    env,
    path::PathBuf,
    process::{Command, Output},
};

use pretty_assertions::assert_eq;

fn check(binary_path: &PathBuf, tree_path: &PathBuf, args: &[&str]) -> Output {
    Command::new(binary_path)
        .arg("check")
        .arg(tree_path)
        .args(args)
        .output()
        .expect("should execute the check command")
}

#[test]
fn checks_invalid_structural_match() {
    let cwd = env::current_dir().unwrap();
    let binary_path = cwd.join("target").join("debug").join("bulloak");
    let tree_path = cwd
        .join("tests")
        .join("check")
        .join("invalid_sol_structure.tree");

    let output = check(&binary_path, &tree_path, &[]);
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
    let binary_path = cwd.join("target").join("debug").join("bulloak");
    let tree_path = cwd.join("tests").join("check").join("extra_codegen.tree");

    let output = check(&binary_path, &tree_path, &[]);
    let actual = String::from_utf8(output.stderr).unwrap();

    // We trim here because we don't care about ending newlines.
    assert_eq!("", actual);
}

#[test]
fn checks_missing_sol_file() {
    let cwd = env::current_dir().unwrap();
    let binary_path = cwd.join("target").join("debug").join("bulloak");
    let tree_path = cwd.join("tests").join("check").join("no_matching_sol.tree");

    let output = check(&binary_path, &tree_path, &[]);
    let actual = String::from_utf8(output.stderr).unwrap();

    let expected = r#"File not found: The file "/Users/alexfertel/code/rust/bulloak/tests/check/no_matching_sol.tree" is missing its matching solidity file."#;

    // We trim here because we don't care about ending newlines.
    assert_eq!(expected.trim(), actual.trim());
}
