//! Violation types for Noir test checking.

use std::fmt;

/// A violation found when checking a Noir test file.
#[derive(Debug, Clone)]
pub struct Violation {
    /// The kind of violation.
    pub kind: ViolationKind,
    /// The file where the violation occurred.
    pub file: String,
}

/// The kind of violation.
#[derive(Debug, Clone)]
pub enum ViolationKind {
    /// The Tree file could not be found.
    TreeFileMissing(String),
    /// The Tree file could not be parsed.
    TreeFileInvalid(String),
    /// The Noir file could not be read. violation.filename is enough to produce an error.
    NoirFileMissing(),
    /// The Noir file could not be parsed.
    NoirFileInvalid(String),
    /// A test function is missing.
    TestFunctionMissing(String),
    /// A setup hook is missing.
    SetupHookMissing(String),
    /// A test function name is present as a setup hook
    TestFunctionWrongType(String),
    /// A setup hook name is present as a test function
    SetupHookWrongType(String),
    /// A test function is present, but in the incorrect place in the noir file.
    TestFunctionWrongPosition(String),
    /// A setup hook is present, but in the incorrect place in the noir file.
    SetupHookWrongPosition(String),
    /// A test should have `#[test(should_fail)]` but doesn't.
    ShouldFailMissing(String),
    /// A test has `#[test(should_fail)]` but shouldn't.
    ShouldFailUnexpected(String),
}

impl Violation {
    /// Create a new violation.
    #[must_use]
    pub fn new(kind: ViolationKind, file: String) -> Self {
        Self { kind, file }
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ViolationKind::TreeFileMissing(err) => {
                write!(f, "bulloak couldn't read the file {}: {}", self.file, err)
            }
            ViolationKind::TreeFileInvalid(err) => {
                write!(f, "Failed to parse tree file {}: {}", self.file, err)
            }
            ViolationKind::NoirFileMissing() => {
                write!(
                    f,
                    "the tree is missing its matching noir file: {}",
                    self.file
                )
            }
            ViolationKind::NoirFileInvalid(err) => {
                write!(f, "Failed to parse Noir file {}: {}", self.file, err)
            }
            ViolationKind::TestFunctionWrongPosition(name) => {
                write!(
                    f,
                    r#"Test function "{}" is in wrong position in {}"#,
                    name, self.file
                )
            }
            ViolationKind::SetupHookWrongPosition(name) => {
                write!(f, r#"Setup hook "{}" is in wrong position in {}"#, name, self.file)
            }
            ViolationKind::TestFunctionMissing(name) => {
                write!(
                    f,
                    r#"Test function "{}" is missing in {}"#,
                    name, self.file
                )
            }
            ViolationKind::SetupHookMissing(name) => {
                write!(f, r#"Missing setup hook "{}" in {}"#, name, self.file)
            }
            ViolationKind::TestFunctionWrongType(name) => {
                write!(
                    f,
                    r#"Test function "{}" is missing its #[test] directive {}"#,
                    name, self.file
                )
            }
            ViolationKind::SetupHookWrongType(name) => {
                write!(f, r#"Setup hook "{}" has unexpected #[test] directive in {}"#, name, self.file)
            }
            ViolationKind::ShouldFailMissing(name) => {
                write!(
                    f,
                    r#"Test "{}" should have #[test(should_fail)] in {}"#,
                    name, self.file
                )
            }
            ViolationKind::ShouldFailUnexpected(name) => {
                write!(
                    f,
                    r#"Test "{}" has #[test(should_fail)] but shouldn't in {}"#,
                    name, self.file
                )
            }
        }
    }
}
