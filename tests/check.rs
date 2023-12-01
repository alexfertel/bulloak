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
    let actual = stderr.lines().filter(|line| line.starts_with("warn:"));

    let expected = r#"warn: function "givenTheStreamIsCold" is missing in .sol
warn: function "whenTheSenderDoesNotRevert" is missing in .sol
warn: incorrect position for function "test_RevertGiven_TheStreamsStatusIsCANCELED"
warn: incorrect position for function "test_RevertGiven_TheStreamsStatusIsSETTLED"
warn: incorrect position for function "test_WhenTheSenderReverts"
warn: incorrect position for function "test_WhenThereIsReentrancy""#
        .lines();

    for (expected, actual) in expected.zip(actual) {
        assert_eq!(expected, actual);
    }
}

#[test]
fn checks_valid_structural_match() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tree_path = cwd
        .join("tests")
        .join("check")
        .join("extra_codegen_sol.tree");

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

    assert!(actual.contains("the tree is missing its matching Solidity file"));
    assert!(actual.contains("no_matching_sol.tree"));
}

#[test]
fn checks_empty_contract() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tree_path = cwd.join("tests").join("check").join("empty_contract.tree");

    let output = cmd(&binary_path, "check", &tree_path, &[]);
    let actual = String::from_utf8(output.stderr).unwrap();

    assert!(actual.contains(r#"function "test_ShouldNeverRevert" is missing in .sol"#));
    assert!(actual.contains(r#"function "test_ShouldNotFindTheSolidityFile" is missing in .sol"#));
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

    assert!(actual.contains(r#"contract "MissingContract" is missing in .sol"#));
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

    assert!(actual.contains(
        r#"contract "ContractName" is missing in .sol -- found "ADifferentName" instead"#
    ));
}

#[test]
fn checks_invalid_tree() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tree_path = cwd.join("tests").join("check").join("invalid.tree");

    let output = cmd(&binary_path, "check", &tree_path, &[]);
    let actual = String::from_utf8(output.stderr).unwrap();

    assert!(actual.contains(r#"an error occurred while parsing the tree: unexpected token '├'"#));
}

#[test]
fn fixes_non_matching_contract_names() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tree_path = cwd
        .join("tests")
        .join("check")
        .join("contract_names_mismatch.tree");

    let output = cmd(&binary_path, "check", &tree_path, &["--fix", "--stdout"]);
    let expected = "// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;

contract ContractName {
    function test_ShouldHaveANameMismatchInTheContracts() external {
        // It should match the result of `keccak256(abi.encodePacked(a,b))`.
    }
}
";

    let actual = String::from_utf8(output.stdout).unwrap();
    assert!(actual.contains(expected));
    assert!(actual.contains("1 issue fixed."));
}

#[test]
fn fixes_contract_missing() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tree_path = cwd
        .join("tests")
        .join("check")
        .join("missing_contract.tree");

    let output = cmd(&binary_path, "check", &tree_path, &["--fix", "--stdout"]);
    let expected = "// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;

contract MissingContract {
    function test_ShouldNotFindAContractInTheSolidityFile() external {
        // It should not find a contract in the Solidity file.
    }
}";

    let actual = String::from_utf8(output.stdout).unwrap();
    assert!(actual.contains(expected));
    assert!(actual.contains("1 issue fixed."));
}

#[test]
fn fixes_extra_codegen_tree() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tree_path = cwd
        .join("tests")
        .join("check")
        .join("extra_codegen_tree.tree");

    let output = cmd(&binary_path, "check", &tree_path, &["--fix", "--stdout"]);
    let actual = String::from_utf8(output.stdout).unwrap();
    assert!(actual.contains("test_ShouldNeverRevert"));
    assert!(actual.contains("Third"));
    assert!(actual.contains("2 issues fixed."));
}

#[test]
fn fixes_invalid_structural_match() {
    let binary_path = get_binary_path();
    let cwd = env::current_dir().unwrap();
    let tree_path = cwd
        .join("tests")
        .join("check")
        .join("invalid_sol_structure.tree");

    let output = cmd(&binary_path, "check", &tree_path, &["--fix", "--stdout"]);
    let actual = String::from_utf8(output.stdout).unwrap();

    let expected = r"// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;

contract CancelTest {
    function test_RevertWhen_DelegateCalled() external {
        // it should revert
    }

    modifier whenNotDelegateCalled() {
        _;
    }

    function test_RevertGiven_TheIdReferencesANullStream() external whenNotDelegateCalled {
        // it should revert
    }

    modifier givenTheIdDoesNotReferenceANullStream() {
        _;
    }

    modifier givenTheStreamIsCold() {
        _;
    }

    function test_RevertGiven_TheStreamsStatusIsDEPLETED()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsCold
    {
        // it should revert
    }

    function test_RevertGiven_TheStreamsStatusIsCANCELED()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsCold
    {
        // it should revert
    }

    function test_RevertGiven_TheStreamsStatusIsSETTLED()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsCold
    {
        // it should revert
    }

    modifier givenTheStreamIsWarm() {
        _;
    }

    modifier whenTheCallerIsAuthorized() {
        _;
    }

    function test_RevertGiven_TheStreamIsNotCancelable()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsWarm
        whenTheCallerIsAuthorized
    {
        // it should revert
    }

    function test_GivenTheSenderIsNotAContract()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsWarm
        whenTheCallerIsAuthorized
    {
        // it should cancel the stream
        // it should mark the stream as canceled
    }

    modifier givenTheSenderIsAContract() {
        _;
    }

    function test_GivenTheSenderDoesNotImplementTheHook()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsWarm
        whenTheCallerIsAuthorized
        givenTheSenderIsAContract
    {
        // it should cancel the stream
        // it should mark the stream as canceled
        // it should call the sender hook
        // it should ignore the revert
    }

    modifier givenTheSenderImplementsTheHook() {
        _;
    }

    function test_WhenTheSenderReverts()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsWarm
        whenTheCallerIsAuthorized
        givenTheSenderIsAContract
        givenTheSenderImplementsTheHook
    {
        // it should cancel the stream
        // it should mark the stream as canceled
        // it should call the sender hook
        // it should ignore the revert
    }

    modifier whenTheSenderDoesNotRevert() {
        _;
    }

    function test_WhenThereIsReentrancy()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsWarm
        whenTheCallerIsAuthorized
        givenTheSenderIsAContract
        givenTheSenderImplementsTheHook
        whenTheSenderDoesNotRevert
    {
        // it should cancel the stream
        // it should mark the stream as canceled
        // it should call the sender hook
        // it should ignore the revert
    }

    function test_WhenThereIsNoReentrancy()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsWarm
        whenTheCallerIsAuthorized
        givenTheSenderIsAContract
        givenTheSenderImplementsTheHook
        whenTheSenderDoesNotRevert
    {
        // it should cancel the stream
        // it should mark the stream as canceled
        // it should make the stream not cancelable
        // it should update the refunded amount
        // it should refund the sender
        // it should call the sender hook
        // it should emit a {MetadataUpdate} event
        // it should emit a {CancelLockupStream} event
    }
}";

    assert!(actual.contains(expected));
    assert!(actual.contains("4 issues fixed."));
}
