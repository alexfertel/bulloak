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
    /// The Noir file could not be parsed.
    NoirFileInvalid(String),
    /// A test function is missing.
    TestFunctionMissing(String),
    /// A helper function is missing.
    HelperFunctionMissing(String),
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
            ViolationKind::NoirFileInvalid(err) => {
                write!(f, "Failed to parse Noir file {}: {}", self.file, err)
            }
            ViolationKind::TestFunctionMissing(name) => {
                write!(f, "Missing test function '{}' in {}", name, self.file)
            }
            ViolationKind::HelperFunctionMissing(name) => {
                write!(f, "Missing helper function '{}' in {}", name, self.file)
            }
            ViolationKind::ShouldFailMissing(name) => {
                write!(
                    f,
                    "Test '{}' should have #[test(should_fail)] in {}",
                    name, self.file
                )
            }
            ViolationKind::ShouldFailUnexpected(name) => {
                write!(
                    f,
                    "Test '{}' has #[test(should_fail)] but shouldn't in {}",
                    name, self.file
                )
            }
        }
    }
}
