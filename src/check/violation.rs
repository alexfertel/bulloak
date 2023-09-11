use std::fmt;

use crate::span::Span;

/// An error that occurred while checking specification rules between
/// a tree and a solidity contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Violation {
    /// The kind of error.
    kind: ViolationKind,
    /// The original text of the tree file.
    text: String,
    /// The span of this error.
    span: Span,
}

impl Violation {
    pub fn new(kind: ViolationKind, span: Span, text: String) -> Self {
        Violation { kind, text, span }
    }

    /// Return the type of this error.
    pub fn kind(&self) -> &ViolationKind {
        &self.kind
    }

    /// The original text string in which this error occurred.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Return the span at which this error occurred.
    pub fn span(&self) -> &Span {
        &self.span
    }
}

/// The type of an error that occurred while checking specification rules between
/// a tree and a solidity contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ViolationKind {
    /// The corresponding solidity file does not exist.
    FileMissing(String),
    /// Couldn't read the corresponding solidity file.
    FileUnreadable(String),
    /// Found a tree branch without a matching test.
    MatchingTestMissing,
    /// This enum may grow additional variants, so this makes sure clients
    /// don't count on exhaustive matching. (Otherwise, adding a new variant
    /// could break existing code.)
    #[doc(hidden)]
    __Nonexhaustive,
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        crate::error::Formatter::from(self).fmt(f)
    }
}

impl fmt::Display for ViolationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::ViolationKind::*;
        match self {
            FileMissing(file) => write!(f, "{} is missing its matching solidity file", file),
            FileUnreadable(file) => write!(f, "failed to read {}", file),
            MatchingTestMissing => write!(f, "the corresponding test is missing"),
            _ => unreachable!(),
        }
    }
}
