//! Structural matching rule for Noir tests.

use std::{collections::HashSet, fs, path::Path};

use anyhow::Result;
use bulloak_syntax::Ast;

use crate::{
    check::violation::{Violation, ViolationKind},
    noir::ParsedNoirFile,
    utils::to_snake_case,
    Config,
};

/// Expected test structure extracted from AST.
struct ExpectedTests {
    helpers: HashSet<String>,
    test_functions: Vec<TestInfo>,
}

struct TestInfo {
    name: String,
    should_fail: bool,
}

/// Check that a Noir test file matches its tree specification.
///
/// # Errors
///
/// Returns an error if checking fails.
pub fn check(tree_path: &Path, cfg: &Config) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();

    // Read the tree file
    let tree_text = fs::read_to_string(tree_path)?;
    let ast = bulloak_syntax::parse_one(&tree_text)?;

    // Find corresponding Noir test file
    let file_stem =
        tree_path.file_stem().and_then(|s| s.to_str()).ok_or_else(|| {
            anyhow::anyhow!("Invalid tree file name: {}", tree_path.display())
        })?;

    let test_file = tree_path.with_file_name(format!("{file_stem}_test.nr"));

    if !test_file.exists() {
        violations.push(Violation::new(
            ViolationKind::NoirFileInvalid(format!(
                "File not found: {}",
                test_file.display()
            )),
            test_file.display().to_string(),
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

    // Extract expected structure from AST
    let expected = extract_expected_structure(&ast, cfg)?;

    // Check helpers (if not skipped)
    if !cfg.skip_setup_hooks {
        let found_helpers = parsed.find_helper_functions();
        let found_helper_set: HashSet<String> =
            found_helpers.into_iter().collect();

        for expected_helper in &expected.helpers {
            if !found_helper_set.contains(expected_helper) {
                violations.push(Violation::new(
                    ViolationKind::HelperFunctionMissing(
                        expected_helper.clone(),
                    ),
                    test_file.display().to_string(),
                ));
            }
        }
    }

    // Check test functions
    let found_tests = parsed.find_test_functions();
    let found_test_map: std::collections::HashMap<String, bool> = found_tests
        .iter()
        .map(|t| (t.name.clone(), t.has_should_fail))
        .collect();

    for expected_test in &expected.test_functions {
        if let Some(&has_should_fail) = found_test_map.get(&expected_test.name)
        {
            // Test exists - check attributes
            if expected_test.should_fail && !has_should_fail {
                violations.push(Violation::new(
                    ViolationKind::ShouldFailMissing(
                        expected_test.name.clone(),
                    ),
                    test_file.display().to_string(),
                ));
            } else if !expected_test.should_fail && has_should_fail {
                violations.push(Violation::new(
                    ViolationKind::ShouldFailUnexpected(
                        expected_test.name.clone(),
                    ),
                    test_file.display().to_string(),
                ));
            }
        } else {
            // Test is missing
            violations.push(Violation::new(
                ViolationKind::TestFunctionMissing(expected_test.name.clone()),
                test_file.display().to_string(),
            ));
        }
    }

    Ok(violations)
}

/// Extract expected test structure from AST.
fn extract_expected_structure(
    ast: &Ast,
    cfg: &Config,
) -> Result<ExpectedTests> {
    let ast_root = match ast {
        Ast::Root(r) => r,
        _ => anyhow::bail!("Expected Root node"),
    };

    let mut helpers = HashSet::new();
    let mut test_functions = Vec::new();

    if !cfg.skip_setup_hooks {
        collect_helpers_recursive(&ast_root.children, &mut helpers);
    }

    collect_tests(&ast_root.children, &[], &mut test_functions, cfg);

    Ok(ExpectedTests { helpers, test_functions })
}

/// Recursively collect helper names from conditions.
fn collect_helpers_recursive(children: &[Ast], helpers: &mut HashSet<String>) {
    for child in children {
        if let Ast::Condition(condition) = child {
            helpers.insert(to_snake_case(&condition.title));
            collect_helpers_recursive(&condition.children, helpers);
        }
    }
}

/// Collect expected test functions.
fn collect_tests(
    children: &[Ast],
    parent_helpers: &[String],
    tests: &mut Vec<TestInfo>,
    cfg: &Config,
) {
    for child in children {
        match child {
            Ast::Condition(condition) => {
                let mut helpers = parent_helpers.to_vec();
                if !cfg.skip_setup_hooks {
                    helpers.push(to_snake_case(&condition.title));
                }

                // Collect all direct Action children
                let actions: Vec<_> = condition
                    .children
                    .iter()
                    .filter_map(|c| match c {
                        Ast::Action(a) => Some(a),
                        _ => None,
                    })
                    .collect();

                // One test function for all actions under this condition
                if !actions.is_empty() {
                    let test_name = if helpers.is_empty() {
                        // Root level action (shouldn't really happen with a
                        // Condition parent, but handle
                        // it just in case)
                        format!("test_{}", to_snake_case(&actions[0].title))
                    } else {
                        // Under conditions: use the last helper name, NOT the
                        // action name
                        format!("test_when_{}", helpers.last().unwrap())
                    };

                    let should_fail =
                        actions.iter().any(|a| has_panic_keyword(&a.title));

                    tests.push(TestInfo { name: test_name, should_fail });
                }

                // Recursively process only nested Condition children (not
                // actions!)
                for child in &condition.children {
                    if matches!(child, Ast::Condition(_)) {
                        collect_tests(
                            std::slice::from_ref(child),
                            &helpers,
                            tests,
                            cfg,
                        );
                    }
                }
            }
            Ast::Action(action) => {
                // Root-level action
                let test_name =
                    format!("test_{}", to_snake_case(&action.title));
                let should_fail = has_panic_keyword(&action.title);
                tests.push(TestInfo { name: test_name, should_fail });
            }
            _ => {}
        }
    }
}

/// Check if a title contains panic keywords.
fn has_panic_keyword(title: &str) -> bool {
    let lower = title.to_lowercase();
    crate::constants::PANIC_KEYWORDS
        .iter()
        .any(|keyword| lower.contains(keyword))
}

#[cfg(test)]
mod tests {
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
}
