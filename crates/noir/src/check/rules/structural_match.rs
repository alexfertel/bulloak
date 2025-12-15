//! Structural matching rule for Noir tests.

use std::{collections::HashSet, fs, path::Path};

use crate::test_structure::{Function, Root, SetupHook};
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
    let parsed = Root { functions: parsed.find_functions() };

    // Extract expected structure from AST
    let expected = Root::new(&forest);
    let comparison_violations = compare_trees(
        &parsed,
        &expected,
        test_file.display().to_string(),
        cfg.skip_setup_hooks,
    );
    violations.extend(comparison_violations);
    Ok(violations)
}

/// iterate over the two trees and report on their differences
fn compare_trees(
    actual: &Root,
    expected: &Root,
    test_file: String,
    skip_setup_hooks: bool,
) -> Vec<Violation> {
    let mut violations = Vec::new();

    let found_tests: std::collections::HashMap<String, bool> = actual
        .functions
        .iter()
        .filter_map(|x| {
            if let Function::TestFunction(t) = x {
                Some((t.name.clone(), t.expect_fail))
            } else {
                None
            }
        })
        .collect();
    let found_hooks: HashSet<SetupHook> = actual
        .functions
        .iter()
        .filter_map(|x| {
            if let Function::SetupHook(h) = x {
                Some(h.clone())
            } else {
                None
            }
        })
        .collect();

    for expected in &expected.functions {
        match expected {
            Function::SetupHook(h) => {
                if !skip_setup_hooks {
                    if !found_hooks.contains(h) {
                        violations.push(Violation::new(
                            ViolationKind::HelperFunctionMissing(
                                h.name.clone(),
                            ),
                            test_file.clone(),
                        ));
                    }
                }
            }
            Function::TestFunction(t) => {
                if let Some(&has_should_fail) = found_tests.get(&t.name) {
                    // TODO: compare invocation of setup hooks and inclusion of action comments
                    let violation_kind = match (t.expect_fail, has_should_fail)
                    {
                        (true, false) => Some(
                            ViolationKind::ShouldFailMissing(t.name.clone()),
                        ),
                        (false, true) => Some(
                            ViolationKind::ShouldFailUnexpected(t.name.clone()),
                        ),
                        _ => None,
                    };
                    if let Some(kind) = violation_kind {
                        violations
                            .push(Violation::new(kind, test_file.clone()));
                    }
                } else {
                    // Test is missing
                    violations.push(Violation::new(
                        ViolationKind::TestFunctionMissing(t.name.clone()),
                        test_file.clone(),
                    ));
                }
            }
        }
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
    fn test_check_passes_when_correct() {
        let tree_content = indoc! {r#"
            hash_pair
            └── It should work.
        "#};

        let noir_content = indoc! {r#"
            // Generated by bulloak

            #[test]
            unconstrained fn test_should_work() {
                // It should work.
            }
        "#};

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

        assert_eq!(violations.len(), 0);

        // Cleanup
        let _ = fs::remove_file(test_path);
    }

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
    use crate::test_structure::TestFunction;

    #[test]
    fn empty_both() {
        let actual = Root { functions: vec![] };
        let expected = Root { functions: vec![] };
        let violations =
            compare_trees(&actual, &expected, "test.nr".to_string(), false);
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
        };
        let expected = Root {
            functions: vec![Function::TestFunction(TestFunction {
                name: "test_foo".to_string(),
                expect_fail: false,
                setup_hooks: vec![],
                actions: vec![],
            })],
        };
        let violations =
            compare_trees(&actual, &expected, "test.nr".to_string(), false);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn missing_test_function() {
        let actual = Root { functions: vec![] };
        let expected = Root {
            functions: vec![Function::TestFunction(TestFunction {
                name: "test_foo".to_string(),
                expect_fail: false,
                setup_hooks: vec![],
                actions: vec![],
            })],
        };
        let violations =
            compare_trees(&actual, &expected, "test.nr".to_string(), false);
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
        let actual = Root { functions: vec![] };
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
        };
        let violations =
            compare_trees(&actual, &expected, "test.nr".to_string(), false);
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
        };
        let expected = Root {
            functions: vec![Function::TestFunction(TestFunction {
                name: "test_foo".to_string(),
                expect_fail: true,
                setup_hooks: vec![],
                actions: vec![],
            })],
        };
        let violations =
            compare_trees(&actual, &expected, "test.nr".to_string(), false);
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
        };
        let expected = Root {
            functions: vec![Function::TestFunction(TestFunction {
                name: "test_foo".to_string(),
                expect_fail: false,
                setup_hooks: vec![],
                actions: vec![],
            })],
        };
        let violations =
            compare_trees(&actual, &expected, "test.nr".to_string(), false);
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
        };
        let expected = Root {
            functions: vec![Function::SetupHook(SetupHook {
                name: "helper_foo".to_string(),
            })],
        };
        let violations =
            compare_trees(&actual, &expected, "test.nr".to_string(), false);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn missing_helper_function() {
        let actual = Root { functions: vec![] };
        let expected = Root {
            functions: vec![Function::SetupHook(SetupHook {
                name: "helper_foo".to_string(),
            })],
        };
        let violations =
            compare_trees(&actual, &expected, "test.nr".to_string(), false);
        assert_eq!(violations.len(), 1);
        assert!(matches!(
            violations[0].kind,
            ViolationKind::HelperFunctionMissing(_)
        ));
        if let ViolationKind::HelperFunctionMissing(name) = &violations[0].kind
        {
            assert_eq!(name, "helper_foo");
        }
    }

    #[test]
    fn skip_setup_hooks_enabled() {
        let actual = Root { functions: vec![] };
        let expected = Root {
            functions: vec![Function::SetupHook(SetupHook {
                name: "helper_foo".to_string(),
            })],
        };
        let violations =
            compare_trees(&actual, &expected, "test.nr".to_string(), true);
        // Should not report missing helper when skip_setup_hooks is true
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn skip_setup_hooks_still_checks_tests() {
        let actual = Root { functions: vec![] };
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
        };
        let violations =
            compare_trees(&actual, &expected, "test.nr".to_string(), true);
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
        };
        let violations =
            compare_trees(&actual, &expected, "test.nr".to_string(), false);
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
        };
        let violations =
            compare_trees(&actual, &expected, "test.nr".to_string(), false);
        assert_eq!(violations.len(), 2);
        assert!(matches!(
            &violations[0].kind,
            ViolationKind::TestFunctionMissing(x) if x == "test_b"
        ));
        assert!(matches!(
            &violations[1].kind,
            ViolationKind::HelperFunctionMissing(x) if x == "helper_a"
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
        };
        let expected = Root {
            functions: vec![Function::TestFunction(TestFunction {
                name: "test_foo".to_string(),
                expect_fail: false,
                setup_hooks: vec![],
                actions: vec![],
            })],
        };
        let violations =
            compare_trees(&actual, &expected, "test.nr".to_string(), false);
        assert_eq!(violations.len(), 0);
    }
}
