//! Defines a rule-checking error object.

use std::fmt;

/// An error that occurred while checking specification rules between
/// a tree and a solidity contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Violation {
    /// The kind of violation.
    kind: ViolationKind,
}

impl Violation {
    /// Create a new violation.
    pub fn new(kind: ViolationKind) -> Self {
        Violation { kind }
    }

    /// Return the type of this violation.
    pub fn kind(&self) -> &ViolationKind {
        &self.kind
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
    /// Found no matching solidity contract.
    ContractMissing(String),
    /// Contract name doesn't match.
    ContractNameNotMatches(String, String),
    /// Found a tree branch without a matching test.
    MatchingTestMissing(String),
    /// Found an incorrectly ordered test.
    TestOrderMismatch(String),
    /// This enum may grow additional variants, so this makes sure clients
    /// don't count on exhaustive matching. (Otherwise, adding a new variant
    /// could break existing code.)
    #[doc(hidden)]
    __Nonexhaustive,
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.kind)
    }
}

impl fmt::Display for ViolationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::ViolationKind::*;
        match self {
            FileMissing(file) => write!(
                f,
                "The file {} is missing its matching solidity file.",
                file
            ),
            FileUnreadable(file) => write!(f, "Bulloak couldn't read {}.", file),
            ContractMissing(contract) => write!(
                f,
                "Couldn't find a corresponding contract for {} in the solidity file.",
                contract
            ),
            MatchingTestMissing(test_name) => write!(
                f,
                "Couldn't find a corresponding test for {} in the solidity file.",
                test_name
            ),
            _ => unreachable!(),
        }
    }
}
