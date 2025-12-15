#![allow(missing_docs)]
use std::env;

use common::{cmd, get_binary_path};
use owo_colors::OwoColorize;
use pretty_assertions::assert_eq;

mod common;

#[test]
fn checks_invalid_structural_match() {
    let binary_path = get_binary_path();
    let cwd = env::current_dir().unwrap();
    let tree_path =
        cwd.join("tests").join("check").join("invalid_sol_structure.tree");

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

    let output = cmd(&binary_path, "check", &tree_path, &["-l", "noir"]);
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Missing setup hook 'given_the_stream_is_cold'"));
    assert!(stderr.contains("Missing setup hook 'when_the_sender_does_not_revert'"));
    assert!(stderr.contains("incorrect position for test function 'test_when_there_is_reentrancy'"));
    assert!(stderr.contains("incorrect position for test function 'test_when_the_sender_reverts'"));
    assert!(stderr.contains("incorrect position for test function 'test_given_the_streams_status_is_canceled'"));
    assert!(stderr.contains("incorrect position for test function 'test_given_the_streams_status_is_settled'"));
    assert!(stderr.contains("invalid_sol_structure_test.nr"));
}

#[test]
fn checks_valid_structural_match() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tree_path =
        cwd.join("tests").join("check").join("extra_codegen_sol.tree");

    let output = cmd(&binary_path, "check", &tree_path, &[]);
    let stderr = String::from_utf8(output.stderr).unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!("", stderr);
    assert!(
        stdout.contains("All checks completed successfully! No issues found.")
    );

    let output = cmd(&binary_path, "check", &tree_path, &["-l", "noir"]);
    let stderr = String::from_utf8(output.stderr).unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!("", stderr);
    assert!(
        stdout.contains("All checks completed successfully! No issues found.")
    );
}

#[test]
fn checks_modifiers_skipped() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tree_path = cwd.join("tests").join("check").join("skip_modifiers.tree");

    let output = cmd(&binary_path, "check", &tree_path, &["-m"]);
    let stderr = String::from_utf8(output.stderr).unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!("", stderr);
    assert!(
        stdout.contains("All checks completed successfully! No issues found.")
    );
}

#[test]
fn checks_modifiers_skipped_issue_81() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tree_path = cwd.join("tests").join("check").join("issue_81.tree");

    let output = cmd(&binary_path, "check", &tree_path, &["-m"]);
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stderr.contains(
        "function \"test_WhenLastUpdatedTimeInPast\" is missing in .sol"
    ));
}

#[test]
fn checks_missing_test_file() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tree_path =
        cwd.join("tests").join("check").join("no_matching_sol.tree");

    let output = cmd(&binary_path, "check", &tree_path, &[]);
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stderr.contains("the tree is missing its matching Solidity file"));
    assert!(stderr.contains("no_matching_sol.tree"));

    let output = cmd(&binary_path, "check", &tree_path, &["-l", "noir"]);
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stderr.contains("the tree is missing its matching noir file"));
    assert!(stderr.contains("no_matching_sol.tree"));
}

#[test]
fn checks_empty_contract() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tree_path = cwd.join("tests").join("check").join("empty_contract.tree");

    let output = cmd(&binary_path, "check", &tree_path, &[]);
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stderr
        .contains(r#"function "test_ShouldNeverRevert" is missing in .sol"#));
    assert!(stderr.contains(
        r#"function "test_ShouldNotFindTheSolidityFile" is missing in .sol"#
    ));

    let output = cmd(&binary_path, "check", &tree_path, &["-l", "noir"]);
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stderr
        .contains(r#"Test function "test_should_never_revert" is missing"#));
    assert!(stderr.contains(
        r#"Test function "test_should_never_revert" is missing"#
    ));
}

#[test]
fn checks_missing_contract() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tree_path =
        cwd.join("tests").join("check").join("missing_contract.tree");

    let output = cmd(&binary_path, "check", &tree_path, &[]);
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stderr.contains(r#"contract "MissingContract" is missing in .sol"#));
}

#[cfg(not(target_os = "windows"))]
#[test]
fn checks_missing_contract_identifier() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tree_path = cwd
        .join("tests")
        .join("check")
        .join("missing_contract_identifier.tree");

    let output = cmd(&binary_path, "check", &tree_path, &[]);
    let stderr = String::from_utf8(output.stderr).unwrap();

    let formatted_message = format!(
        "{}: an error occurred while parsing the tree: contract name missing at tree root #2\n   {} {}",
        "warn".yellow(),
        "-->".blue(),
        tree_path.display()
    );

    assert!(
        stderr.contains(&formatted_message),
        "stderr: {}\nmessage: {}",
        stderr,
        formatted_message
    );
}

#[test]
fn checks_contract_name_mismatch() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tree_path =
        cwd.join("tests").join("check").join("contract_names_mismatch.tree");

    let output = cmd(&binary_path, "check", &tree_path, &[]);
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stderr.contains(
        r#"contract "ContractName" is missing in .sol -- found "ADifferentName" instead"#
    ));
}

#[cfg(not(target_os = "windows"))]
#[test]
fn checks_contract_name_mismatch_multiple_roots() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tree_path = cwd
        .join("tests")
        .join("check")
        .join("contract_names_mismatch_multiple_roots.tree");

    let output = cmd(&binary_path, "check", &tree_path, &[]);
    let stderr = String::from_utf8(output.stderr).unwrap();

    let formatted_message = format!(
        "{}: an error occurred while parsing the tree: contract name mismatch: expected 'ContractName', found 'MismatchedContractName'\n   {} {}",
        "warn".yellow(),
        "-->".blue(),
        tree_path.display()
    );

    assert!(stderr.contains(&formatted_message));
}

#[test]
fn checks_invalid_tree() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tree_path = cwd.join("tests").join("check").join("invalid.tree");

    let output = cmd(&binary_path, "check", &tree_path, &[]);
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stderr.contains(
        r#"an error occurred while parsing the tree: unexpected token '├'"#
    ));

    let output = cmd(&binary_path, "check", &tree_path, &["-l", "noir"]);
    let stderr = String::from_utf8(output.stderr).unwrap();

    // it's okay to be less specific with error messages for now
    assert!(stderr.contains(r#"bulloak error: unexpected token '├'"#));
    assert!(stderr.contains(r#"Failed to parse tree file"#));
}

#[test]
fn fixes_non_matching_contract_names() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tree_path =
        cwd.join("tests").join("check").join("contract_names_mismatch.tree");

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
    let tree_path =
        cwd.join("tests").join("check").join("missing_contract.tree");

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
    let tree_path =
        cwd.join("tests").join("check").join("extra_codegen_tree.tree");

    let output = cmd(&binary_path, "check", &tree_path, &["--fix", "--stdout"]);
    let actual = String::from_utf8(output.stdout).unwrap();
    assert!(actual.contains("test_ShouldNeverRevert"));
    assert!(actual.contains("Third"));
    assert!(actual.contains("2 issues fixed."));
}

#[test]
fn fixes_extra_fn_plus_wrong_order() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tree_path =
        cwd.join("tests").join("check").join("fix_extra_fn_plus_order.tree");

    let output = cmd(&binary_path, "check", &tree_path, &["--fix", "--stdout"]);
    let actual = String::from_utf8(output.stdout).unwrap();

    let expected = r"// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;

contract Foo {
    function test_WhenB() external {
        // it Y
    }

    function test_WhenA() external {
        // it X
    }

    // <<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<
    // ==================== BULLOAK AUTOGENERATED SEPARATOR ====================
    // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
    //    Code below this section could not be automatically moved by bulloak
    // =========================================================================

    function test_WhenTheMethodIsCalledASecondTime() external {
        // it Z
    }
}";

    assert!(actual.contains(expected));
    assert!(actual.contains("1 issue fixed."));
}

#[test]
fn fixes_invalid_structural_match() {
    let binary_path = get_binary_path();
    let cwd = env::current_dir().unwrap();
    let tree_path =
        cwd.join("tests").join("check").join("invalid_sol_structure.tree");

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

/// If you pass an invalid glob to `bulloak check`,
/// it should warn but still exit code = 0 and report “no issues found.”
#[test]
fn check_invalid_glob_warns_but_reports_success() {
    let cwd = env::current_dir().unwrap();
    let bin = common::get_binary_path();

    let bad_glob = cwd.join("tests").join("check").join("*[.tree");
    let out = cmd(&bin, "check", &bad_glob, &[]);

    assert!(
        out.status.success(),
        "check should succeed even on invalid glob, got {:?}",
        out
    );

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("could not expand"),
        "did not see warn on stderr: {}",
        stderr
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("All checks completed successfully"),
        "expected success message, got: {}",
        stdout
    );

    let bad_glob = cwd.join("tests").join("check").join("*[.tree");
    let out = cmd(&bin, "check", &bad_glob, &["-l", "noir"]);

    assert!(
        out.status.success(),
        "check should succeed even on invalid glob, got {:?}",
        out
    );

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("could not expand"),
        "did not see warn on stderr: {}",
        stderr
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("All checks completed successfully"),
        "expected success message, got: {}",
        stdout
    );
}
