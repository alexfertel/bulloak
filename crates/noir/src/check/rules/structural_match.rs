//! Structural matching rule for Noir tests.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use crate::{
    test_structure::{Function, Root},
    utils::get_module_name,
};
use anyhow::Result;

use crate::{
    check::violation::{Violation, ViolationKind},
    noir::ParsedNoirFile,
    Config,
};

/// Check that a Noir test file matches its tree specification.
///
/// # Errors
///
/// Returns an error if checking fails.
pub fn check(tree_path: &Path, cfg: &Config) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();

    let tree_text = match fs::read_to_string(tree_path) {
        Err(e) => {
            violations.push(Violation::new(
                ViolationKind::TreeFileMissing(e.to_string()),
                tree_path.display().to_string(),
            ));
            return Ok(violations);
        }
        Ok(a) => a,
    };
    let forest = match bulloak_syntax::parse(&tree_text) {
        Err(e) => {
            violations.push(Violation::new(
                ViolationKind::TreeFileInvalid(format!(
                    "an error occurred while parsing the tree: {}",
                    e
                )),
                tree_path.display().to_string(),
            ));
            return Ok(violations);
        }
        Ok(a) => a,
    };

    // Find corresponding Noir test file
    // TODO don't bother reusing the test_filename function here since after the Big Backend
    // Refactor this module won't know about filenames
    let file_stem = tree_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_else(|| panic!("this condition should be unreachable, as the file was successfully read once already."));

    match get_module_name(&forest) {
        None => {} // empty tree
        // tree has only one root, check if it matches the filename
        Some(Ok(module)) => {
            if module != file_stem {
                violations.push(Violation::new(
                    ViolationKind::TreeFileWrongRoot(
                        module.to_string(),
                        file_stem.to_string(),
                    ),
                    tree_path.display().to_string(),
                ));
            }
        }
        Some(Err((expected, second))) => {
            violations.push(
                Violation::new(
ViolationKind::TreeFileInvalid(format!(
                        "an error occurred while parsing the tree: module name mismatch: expected '{}', found '{}'",
                        expected, second
                    )),
                tree_path.display().to_string(),
            )
            );
            return Ok(violations);
        }
    }

    let test_file = tree_path.with_file_name(format!("{file_stem}_test.nr"));

    if !test_file.exists() {
        violations.push(Violation::new(
            ViolationKind::NoirFileMissing(),
            tree_path.display().to_string(),
        ));
        return Ok(violations);
    }

    let noir_source = fs::read_to_string(&test_file)?;

    // Parse the Noir file
    let parsed = match ParsedNoirFile::parse(&noir_source) {
        Ok(p) => p,
        Err(e) => {
            violations.push(Violation::new(
                ViolationKind::NoirFileInvalid(e.to_string()),
                test_file.display().to_string(),
            ));
            return Ok(violations);
        }
    };
    let parsed = Root { functions: parsed.find_functions(), modules: vec![] };

    // An AST may be valid syntactically but not semantically,
    // in which case we cannot produce a testfile structure from it
    let expected = match Root::new(&forest) {
        Ok(r) => r,
        Err(e) => {
            violations.push(Violation::new(
                ViolationKind::TreeFileInvalid(e.to_string()),
                tree_path.display().to_string(),
            ));
            return Ok(violations);
        }
    };
    let comparison_violations = compare_trees(
        parsed,
        expected,
        test_file.display().to_string(),
        cfg.skip_setup_hooks,
    );
    violations.extend(comparison_violations);
    Ok(violations)
}

/// iterate over the two trees and report on their differences
fn compare_trees(
    actual: Root,
    expected: Root,
    test_file: String,
    skip_setup_hooks: bool,
) -> Vec<Violation> {
    let mut violations = Vec::new();
    let expected_set: BTreeSet<String> =
        expected.functions.iter().map(|x| x.name()).collect();

    // name -> (full obj, index)
    let found_fns: BTreeMap<String, (Function, usize)> = actual
        .functions
        .into_iter()
        // should I define a custom Hash implementation that hashes the name only?
        .filter(|x| expected_set.contains(&x.name()))
        .enumerate() // indices within the set of functions that we care about, not within all functions
        .map(|(k, v)| (v.name(), (v, k)))
        .collect();

    for expected in &expected.functions {
        let found;
        if let Some((f, _)) = found_fns.get(&expected.name()) {
            found = f;
        } else {
            match expected {
                Function::SetupHook(_) => {
                    if !skip_setup_hooks {
                        violations.push(Violation::new(
                            ViolationKind::SetupHookMissing(expected.name()),
                            test_file.clone(),
                        ));
                    }
                }
                Function::TestFunction(_) => {
                    violations.push(Violation::new(
                        ViolationKind::TestFunctionMissing(expected.name()),
                        test_file.clone(),
                    ));
                }
            }
            continue;
        }

        match (expected, found) {
            (Function::SetupHook(_), Function::TestFunction(_)) => violations
                .push(Violation::new(
                    ViolationKind::SetupHookWrongType(expected.name()),
                    test_file.clone(),
                )),
            (Function::TestFunction(_), Function::SetupHook(_)) => violations
                .push(Violation::new(
                    ViolationKind::TestFunctionWrongType(expected.name()),
                    test_file.clone(),
                )),
            // setup hooks dont really have any attributes and we are not comparing order yet
            (Function::SetupHook(_), Function::SetupHook(_)) => {}
            (
                Function::TestFunction(expected),
                Function::TestFunction(found),
            ) => {
                // TODO: compare invocation of setup hooks and inclusion of action comments
                // (not present in foundry backend but would be cool)
                let violation_kind =
                    match (expected.expect_fail, found.expect_fail) {
                        (true, false) => {
                            Some(ViolationKind::ShouldFailMissing(
                                expected.name.clone(),
                            ))
                        }
                        (false, true) => {
                            Some(ViolationKind::ShouldFailUnexpected(
                                expected.name.clone(),
                            ))
                        }
                        _ => None,
                    };
                if let Some(kind) = violation_kind {
                    violations.push(Violation::new(kind, test_file.clone()));
                }
            }
        }
    }

    let present_expected_fns: BTreeMap<String, (Function, usize)> = expected
        .functions
        .iter()
        .filter(|x| found_fns.contains_key(&x.name()))
        .enumerate()
        .map(|(i, v)| (v.name(), (v.clone(), i)))
        .collect();
    for (name, (expected, expected_index)) in present_expected_fns {
        let (found, found_index) = found_fns
            .get(&name)
            .unwrap_or_else(|| panic!("just filtered for this!"));

        match (expected.clone(), found) {
            (Function::TestFunction(_), Function::TestFunction(_))
            | (Function::SetupHook(_), Function::SetupHook(_)) => {
                if *found_index != expected_index {
                    violations.push(Violation::new(
                        match expected {
                            Function::SetupHook(_) => {
                                ViolationKind::SetupHookWrongPosition(name)
                            }
                            Function::TestFunction(_) => {
                                ViolationKind::TestFunctionWrongPosition(name)
                            }
                        },
                        test_file.clone(),
                    ));
                }
            }
            // Already handled when searching for wrong type/missing fns above
            // we can't really comment on the ordering of a function that is of a wrong type
            _ => {}
        };
    }
    violations
}

#[cfg(test)]
mod check_test {
    use std::io::Write;

    use indoc::indoc;
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test_check_fails_when_missing_test() {
        let tree_content = indoc! {r#"
            test_root
            └── It should work.
        "#};

        let noir_content = "// Generated by bulloak\n\n";

        let mut tree_file = NamedTempFile::new().unwrap();
        tree_file.write_all(tree_content.as_bytes()).unwrap();
        tree_file.flush().unwrap();

        let test_path = tree_file.path().with_file_name(format!(
            "{}_test.nr",
            tree_file.path().file_stem().unwrap().to_str().unwrap()
        ));
        fs::write(&test_path, noir_content).unwrap();

        let cfg = Config::default();
        let violations = check(tree_file.path(), &cfg).unwrap();

        assert!(violations.len() > 0);
        assert!(violations
            .iter()
            .any(|v| matches!(v.kind, ViolationKind::TestFunctionMissing(_))));

        // Cleanup
        let _ = fs::remove_file(test_path);
    }

    #[test]
    fn test_check_fails_when_missing_spec() {
        let cfg = Config::default();
        let violations = check(Path::new("not_there.tree"), &cfg).unwrap();

        assert!(violations.len() == 1);
        assert!(matches!(
            violations[0].kind,
            ViolationKind::TreeFileMissing(_)
        ));
    }

    #[test]
    fn test_check_fails_when_empty_spec_filename() {
        let cfg = Config::default();
        let violations = check(Path::new(""), &cfg).unwrap();

        assert!(violations.len() == 1);
        assert!(matches!(
            violations[0].kind,
            ViolationKind::TreeFileMissing(_)
        ));
    }
}

#[cfg(test)]
mod compare_trees_test {
    use super::*;
    use crate::test_structure::{SetupHook, TestFunction};

    #[test]
    fn empty_both() {
        let actual = Root { functions: vec![], modules: vec![] };
        let expected = Root { functions: vec![], modules: vec![] };
        let violations =
            compare_trees(actual, expected, "test.nr".to_string(), false);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn matching_single_test() {
        let actual = Root {
            functions: vec![Function::TestFunction(TestFunction {
                name: "test_foo".to_string(),
                expect_fail: false,
                setup_hooks: vec![],
                actions: vec![],
            })],
            modules: vec![],
        };
        let expected = Root {
            functions: vec![Function::TestFunction(TestFunction {
                name: "test_foo".to_string(),
                expect_fail: false,
                setup_hooks: vec![],
                actions: vec![],
            })],
            modules: vec![],
        };
        let violations =
            compare_trees(actual, expected, "test.nr".to_string(), false);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn missing_test_function() {
        let actual = Root { functions: vec![], modules: vec![] };
        let expected = Root {
            functions: vec![Function::TestFunction(TestFunction {
                name: "test_foo".to_string(),
                expect_fail: false,
                setup_hooks: vec![],
                actions: vec![],
            })],
            modules: vec![],
        };
        let violations =
            compare_trees(actual, expected, "test.nr".to_string(), false);
        assert_eq!(violations.len(), 1);
        assert!(matches!(
            violations[0].kind,
            ViolationKind::TestFunctionMissing(_)
        ));
        if let ViolationKind::TestFunctionMissing(name) = &violations[0].kind {
            assert_eq!(name, "test_foo");
        }
    }

    #[test]
    fn missing_multiple_tests() {
        let actual = Root { functions: vec![], modules: vec![] };
        let expected = Root {
            functions: vec![
                Function::TestFunction(TestFunction {
                    name: "test_foo".to_string(),
                    expect_fail: false,
                    setup_hooks: vec![],
                    actions: vec![],
                }),
                Function::TestFunction(TestFunction {
                    name: "test_bar".to_string(),
                    expect_fail: false,
                    setup_hooks: vec![],
                    actions: vec![],
                }),
            ],
            modules: vec![],
        };
        let violations =
            compare_trees(actual, expected, "test.nr".to_string(), false);
        assert_eq!(violations.len(), 2);
        assert!(violations
            .iter()
            .all(|v| matches!(v.kind, ViolationKind::TestFunctionMissing(_))));
    }

    #[test]
    fn should_fail_missing() {
        let actual = Root {
            functions: vec![Function::TestFunction(TestFunction {
                name: "test_foo".to_string(),
                expect_fail: false,
                setup_hooks: vec![],
                actions: vec![],
            })],
            modules: vec![],
        };
        let expected = Root {
            functions: vec![Function::TestFunction(TestFunction {
                name: "test_foo".to_string(),
                expect_fail: true,
                setup_hooks: vec![],
                actions: vec![],
            })],
            modules: vec![],
        };
        let violations =
            compare_trees(actual, expected, "test.nr".to_string(), false);
        assert_eq!(violations.len(), 1);
        assert!(matches!(
            violations[0].kind,
            ViolationKind::ShouldFailMissing(_)
        ));
        if let ViolationKind::ShouldFailMissing(name) = &violations[0].kind {
            assert_eq!(name, "test_foo");
        }
    }

    #[test]
    fn should_fail_unexpected() {
        let actual = Root {
            functions: vec![Function::TestFunction(TestFunction {
                name: "test_foo".to_string(),
                expect_fail: true,
                setup_hooks: vec![],
                actions: vec![],
            })],
            modules: vec![],
        };
        let expected = Root {
            functions: vec![Function::TestFunction(TestFunction {
                name: "test_foo".to_string(),
                expect_fail: false,
                setup_hooks: vec![],
                actions: vec![],
            })],
            modules: vec![],
        };
        let violations =
            compare_trees(actual, expected, "test.nr".to_string(), false);
        assert_eq!(violations.len(), 1);
        assert!(matches!(
            violations[0].kind,
            ViolationKind::ShouldFailUnexpected(_)
        ));
        if let ViolationKind::ShouldFailUnexpected(name) = &violations[0].kind {
            assert_eq!(name, "test_foo");
        }
    }

    #[test]
    fn matching_helper_function() {
        let actual = Root {
            functions: vec![Function::SetupHook(SetupHook {
                name: "helper_foo".to_string(),
            })],
            modules: vec![],
        };
        let expected = Root {
            functions: vec![Function::SetupHook(SetupHook {
                name: "helper_foo".to_string(),
            })],
            modules: vec![],
        };
        let violations =
            compare_trees(actual, expected, "test.nr".to_string(), false);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn missing_helper_function() {
        let actual = Root { functions: vec![], modules: vec![] };
        let expected = Root {
            functions: vec![Function::SetupHook(SetupHook {
                name: "helper_foo".to_string(),
            })],
            modules: vec![],
        };
        let violations =
            compare_trees(actual, expected, "test.nr".to_string(), false);
        assert_eq!(violations.len(), 1);
        assert!(matches!(
            violations[0].kind,
            ViolationKind::SetupHookMissing(_)
        ));
        if let ViolationKind::SetupHookMissing(name) = &violations[0].kind {
            assert_eq!(name, "helper_foo");
        }
    }

    #[test]
    fn skip_setup_hooks_enabled() {
        let actual = Root { functions: vec![], modules: vec![] };
        let expected = Root {
            functions: vec![Function::SetupHook(SetupHook {
                name: "helper_foo".to_string(),
            })],
            modules: vec![],
        };
        let violations =
            compare_trees(actual, expected, "test.nr".to_string(), true);
        // Should not report missing helper when skip_setup_hooks is true
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn skip_setup_hooks_still_checks_tests() {
        let actual = Root { functions: vec![], modules: vec![] };
        let expected = Root {
            functions: vec![
                Function::SetupHook(SetupHook {
                    name: "helper_foo".to_string(),
                }),
                Function::TestFunction(TestFunction {
                    name: "test_bar".to_string(),
                    expect_fail: false,
                    setup_hooks: vec![],
                    actions: vec![],
                }),
            ],
            modules: vec![],
        };
        let violations =
            compare_trees(actual, expected, "test.nr".to_string(), true);
        // Should still report missing test even with skip_setup_hooks
        assert_eq!(violations.len(), 1);
        assert!(matches!(
            violations[0].kind,
            ViolationKind::TestFunctionMissing(_)
        ));
    }

    #[test]
    fn mixed_functions() {
        let actual = Root {
            functions: vec![
                Function::SetupHook(SetupHook { name: "helper_a".to_string() }),
                Function::TestFunction(TestFunction {
                    name: "test_b".to_string(),
                    expect_fail: false,
                    setup_hooks: vec![],
                    actions: vec![],
                }),
            ],
            modules: vec![],
        };
        let expected = Root {
            functions: vec![
                Function::SetupHook(SetupHook { name: "helper_a".to_string() }),
                Function::TestFunction(TestFunction {
                    name: "test_b".to_string(),
                    expect_fail: false,
                    setup_hooks: vec![],
                    actions: vec![],
                }),
            ],
            modules: vec![],
        };
        let violations =
            compare_trees(actual, expected, "test.nr".to_string(), false);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn incorrect_type() {
        let actual = Root {
            functions: vec![
                Function::SetupHook(SetupHook { name: "test_b".to_string() }),
                Function::TestFunction(TestFunction {
                    name: "helper_a".to_string(),
                    expect_fail: false,
                    setup_hooks: vec![],
                    actions: vec![],
                }),
            ],
            modules: vec![],
        };
        let expected = Root {
            functions: vec![
                Function::SetupHook(SetupHook { name: "helper_a".to_string() }),
                Function::TestFunction(TestFunction {
                    name: "test_b".to_string(),
                    expect_fail: false,
                    setup_hooks: vec![],
                    actions: vec![],
                }),
            ],
            modules: vec![],
        };
        let violations =
            compare_trees(actual, expected, "test.nr".to_string(), false);
        assert_eq!(violations.len(), 2);
        assert!(matches!(
            &violations[0].kind,
            ViolationKind::SetupHookWrongType(x) if x == "helper_a"
        ));
        assert!(matches!(
            &violations[1].kind,
            ViolationKind::TestFunctionWrongType(x) if x == "test_b"
        ));
    }

    #[test]
    fn extra_functions_in_actual() {
        // Extra functions in actual should not cause violations
        // (compare_trees only checks that expected functions are present)
        let actual = Root {
            functions: vec![
                Function::TestFunction(TestFunction {
                    name: "test_foo".to_string(),
                    expect_fail: false,
                    setup_hooks: vec![],
                    actions: vec![],
                }),
                Function::TestFunction(TestFunction {
                    name: "test_extra".to_string(),
                    expect_fail: false,
                    setup_hooks: vec![],
                    actions: vec![],
                }),
            ],
            modules: vec![],
        };
        let expected = Root {
            functions: vec![Function::TestFunction(TestFunction {
                name: "test_foo".to_string(),
                expect_fail: false,
                setup_hooks: vec![],
                actions: vec![],
            })],
            modules: vec![],
        };
        let violations =
            compare_trees(actual, expected, "test.nr".to_string(), false);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn ordering_inverted() {
        let actual = Root {
            functions: vec![
                Function::TestFunction(TestFunction {
                    name: "test_b".to_string(),
                    expect_fail: false,
                    setup_hooks: vec![],
                    actions: vec![],
                }),
                Function::SetupHook(SetupHook { name: "helper_a".to_string() }),
            ],
            modules: vec![],
        };
        let expected = Root {
            functions: vec![
                Function::SetupHook(SetupHook { name: "helper_a".to_string() }),
                Function::TestFunction(TestFunction {
                    name: "test_b".to_string(),
                    expect_fail: false,
                    setup_hooks: vec![],
                    actions: vec![],
                }),
            ],
            modules: vec![],
        };
        let violations =
            compare_trees(actual, expected, "test.nr".to_string(), false);
        assert_eq!(violations.len(), 2);
        assert!(matches!(
            &violations[0].kind,
            ViolationKind::SetupHookWrongPosition(x) if x == "helper_a"
        ));
        assert!(matches!(
            &violations[1].kind,
            ViolationKind::TestFunctionWrongPosition(x) if x == "test_b"
        ));
    }

    #[test]
    fn ordering_incorrect_with_extra_function() {
        let actual = Root {
            functions: vec![
                Function::SetupHook(SetupHook {
                    name: "other_fun".to_string(),
                }),
                Function::TestFunction(TestFunction {
                    name: "other_test".to_string(),
                    expect_fail: false,
                    setup_hooks: vec![],
                    actions: vec![],
                }),
                Function::TestFunction(TestFunction {
                    name: "test_b".to_string(),
                    expect_fail: false,
                    setup_hooks: vec![],
                    actions: vec![],
                }),
                Function::TestFunction(TestFunction {
                    name: "other_test".to_string(),
                    expect_fail: false,
                    setup_hooks: vec![],
                    actions: vec![],
                }),
                Function::SetupHook(SetupHook { name: "helper_a".to_string() }),
            ],
            modules: vec![],
        };
        let expected = Root {
            functions: vec![
                Function::SetupHook(SetupHook { name: "helper_a".to_string() }),
                Function::TestFunction(TestFunction {
                    name: "test_b".to_string(),
                    expect_fail: false,
                    setup_hooks: vec![],
                    actions: vec![],
                }),
            ],
            modules: vec![],
        };
        let violations =
            compare_trees(actual, expected, "test.nr".to_string(), false);
        assert_eq!(violations.len(), 2);
        assert!(matches!(
            &violations[0].kind,
            ViolationKind::SetupHookWrongPosition(x) if x == "helper_a"
        ));
        assert!(matches!(
            &violations[1].kind,
            ViolationKind::TestFunctionWrongPosition(x) if x == "test_b"
        ));
    }

    #[test]
    fn ordering_correct_with_extra_function() {
        let actual = Root {
            functions: vec![
                Function::SetupHook(SetupHook {
                    name: "other_fun".to_string(),
                }),
                Function::SetupHook(SetupHook { name: "helper_a".to_string() }),
                Function::TestFunction(TestFunction {
                    name: "other_test".to_string(),
                    expect_fail: false,
                    setup_hooks: vec![],
                    actions: vec![],
                }),
                Function::TestFunction(TestFunction {
                    name: "test_b".to_string(),
                    expect_fail: false,
                    setup_hooks: vec![],
                    actions: vec![],
                }),
                Function::TestFunction(TestFunction {
                    name: "other_test".to_string(),
                    expect_fail: false,
                    setup_hooks: vec![],
                    actions: vec![],
                }),
            ],
            modules: vec![],
        };
        let expected = Root {
            functions: vec![
                Function::SetupHook(SetupHook { name: "helper_a".to_string() }),
                Function::TestFunction(TestFunction {
                    name: "test_b".to_string(),
                    expect_fail: false,
                    setup_hooks: vec![],
                    actions: vec![],
                }),
            ],
            modules: vec![],
        };
        let violations =
            compare_trees(actual, expected, "test.nr".to_string(), false);
        assert_eq!(violations.len(), 0);
    }
}
