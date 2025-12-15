//! Structural matching rule for Noir tests.

use std::{collections::HashSet, fs, path::Path};

use crate::test_structure::{Root, SetupHook};
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
    let parsed = Root {
        setup_hooks: parsed.find_helper_functions(),
        tests: parsed.find_test_functions(),
    };

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
    // Check helpers (if not skipped)
    if !skip_setup_hooks {
        let found_helpers = actual.setup_hooks.clone();
        let found_helper_set: HashSet<SetupHook> =
            found_helpers.into_iter().collect();

        for expected_helper in &expected.setup_hooks {
            if !found_helper_set.contains(expected_helper) {
                violations.push(Violation::new(
                    ViolationKind::HelperFunctionMissing(
                        expected_helper.name.clone(),
                    ),
                    test_file.clone(),
                ));
            }
        }
    }

    // Check test functions
    let found_tests = actual.tests.clone();
    let found_test_map: std::collections::HashMap<String, bool> =
        found_tests.iter().map(|t| (t.name.clone(), t.expect_fail)).collect();

    for expected_test in &expected.tests {
        if let Some(&has_should_fail) = found_test_map.get(&expected_test.name)
        {
            // TODO: compare invocation of setup hooks and inclusion of action comments
            // Test exists - check attributes
            let violation_kind =
                match (expected_test.expect_fail, has_should_fail) {
                    (true, false) => Some(ViolationKind::ShouldFailMissing(
                        expected_test.name.clone(),
                    )),
                    (false, true) => Some(ViolationKind::ShouldFailUnexpected(
                        expected_test.name.clone(),
                    )),
                    _ => None,
                };
            if let Some(kind) = violation_kind {
                violations.push(Violation::new(kind, test_file.clone()));
            }
        } else {
            // Test is missing
            violations.push(Violation::new(
                ViolationKind::TestFunctionMissing(expected_test.name.clone()),
                test_file.clone(),
            ));
        }
    }

    violations
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
