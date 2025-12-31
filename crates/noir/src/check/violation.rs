//! Violation types for Noir test checking.

use std::fmt;

use owo_colors::OwoColorize;

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
    /// This error is produced when processing the Noir file although it could be detected when
    /// processing the tree file, since this is specific to Noir's constraints
    TreeFileWrongRoot(String, String),
    /// A test function is missing.
    TestFunctionMissing(String, String),
    /// A setup hook is missing.
    SetupHookMissing(String, String),
    /// A module is missing from the noir file
    ModuleMissing(String),
    /// A test function name is present as a setup hook
    TestFunctionWrongType(String, String),
    /// A setup hook name is present as a test function
    SetupHookWrongType(String, String),
    /// A test function is present, but in the incorrect place in the noir file.
    TestFunctionWrongPosition(String, String),
    /// A setup hook is present, but in the incorrect place in the noir file.
    SetupHookWrongPosition(String, String),
    /// A module is present, but in the incorrect place in the noir file.
    ModuleWrongPosition(String),
    /// A test should have `#[test(should_fail)]` but doesn't.
    ShouldFailMissing(String, String),
    /// A test has `#[test(should_fail)]` but shouldn't.
    ShouldFailUnexpected(String, String),
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
                write!(
                    f,
                    "bulloak couldn't read the file {}: {}",
                    self.file, err
                )
            }
            ViolationKind::TreeFileInvalid(err) => {
                writeln!(f, "{}: {}", "warn".yellow(), err)?;
                write!(f, "   {} {}", "-->".blue(), self.file)
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
            ViolationKind::TreeFileWrongRoot(actual, expected) => {
                write!(
                    f,
                    r#"Tree root "{}" should match treefile name: "{}""#,
                    actual, expected
                )
            }
            ViolationKind::TestFunctionWrongPosition(name, module) => {
                write!(
                    f,
                    r#"Test function "{}" is in wrong position in module {} in file {}"#,
                    name, module, self.file
                )
            }
            ViolationKind::SetupHookWrongPosition(name, module) => {
                write!(
                    f,
                    r#"Setup hook "{}" is in wrong position in module {} in file {}"#,
                    name, module, self.file
                )
            }
            ViolationKind::ModuleWrongPosition(name) => {
                write!(
                    f,
                    r#"Module "{}" is in wrong position in {}"#,
                    name, self.file
                )
            }
            ViolationKind::TestFunctionMissing(name, module) => {
                write!(
                    f,
                    r#"Test function "{}" is missing in module {} in file {}"#,
                    name, module, self.file
                )
            }
            ViolationKind::SetupHookMissing(name, module) => {
                write!(f, r#"Missing setup hook "{}" in module {} in file {}"#,
                    name, module, self.file)
            }
            ViolationKind::ModuleMissing(name) => {
                write!(f, r#"Module "{}" is missing in {}"#,
                    name, self.file)
            }
            ViolationKind::TestFunctionWrongType(name, module) => {
                write!(
                    f,
                    r#"Test function "{}" is missing its #[test] directive in module {} in file {}"#,
                    name, module, self.file
                )
            }
            ViolationKind::SetupHookWrongType(name, module) => {
                write!(
                    f,
                    r#"Setup hook "{}" has unexpected #[test] directive in module {} in file {}"#,
                    name, module, self.file
                )
            }
            ViolationKind::ShouldFailMissing(name, module) => {
                write!(
                    f,
                    r#"Test "{}" should have #[test(should_fail)] in module {} in file {}"#,
                    name, module, self.file
                )
            }
            ViolationKind::ShouldFailUnexpected(name, module) => {
                write!(
                    f,
                    r#"Test "{}" has #[test(should_fail)] but shouldn't in module {} in file {}"#,
                    name, module, self.file
                )
            }
        }
    }
}
