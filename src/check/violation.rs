//! Defines a rule-checking error object.

use std::fmt;

/// An error that occurred while checking specification rules between
/// a tree and a solidity contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Violation {
    /// The kind of violation.
    kind: ViolationKind,
}

impl Violation {
    /// Create a new violation.
    pub(crate) fn new(kind: ViolationKind) -> Self {
        Violation { kind }
    }
}

/// The type of an error that occurred while checking specification rules between
/// a tree and a solidity contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ViolationKind {
    /// The corresponding solidity file does not exist.
    FileMissing(String),
    /// Couldn't read the corresponding solidity file.
    FileUnreadable(String),
    /// Found no matching solidity contract.
    ContractMissing(String),
    /// Contract name doesn't match.
    ContractNameNotMatches(String, String),
    /// Found a tree element without its matching codegen.
    MatchingCodegenMissing(String),
    /// Found an incorrectly ordered element.
    CodegenOrderMismatch(String),
    /// This enum may grow additional variants, so this makes sure clients
    /// don't count on exhaustive matching. (Otherwise, adding a new variant
    /// could break existing code.)
    #[doc(hidden)]
    __Nonexhaustive,
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl fmt::Display for ViolationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::ViolationKind::*;
        match self {
            FileMissing(file) => write!(
                f,
                r#"File not found: The file "{}" is missing its matching solidity file."#,
                file
            ),
            FileUnreadable(file) => {
                write!(f, r#"File unreadable: Bulloak couldn't read "{}"."#, file)
            }
            ContractMissing(contract) => write!(
                f,
                r#"Contract not found: Couldn't find a corresponding contract for "{}" in the solidity file."#,
                contract
            ),
            MatchingCodegenMissing(codegen_name) => write!(
                f,
                r#"Codegen not found: Couldn't find a corresponding element for "{}" in the solidity file."#,
                codegen_name
            ),
            ContractNameNotMatches(tree_name, sol_name) => write!(
                f,
                r#"Invalid contract name: Couldn't find a corresponding contract for "{}" in the solidity file. Found "{}"."#,
                tree_name, sol_name
            ),
            CodegenOrderMismatch(codegen_name) => write!(
                f,
                r#"Invalid codegen order: Found a matching element for "{}", but the order is not correct."#,
                codegen_name
            ),
            _ => unreachable!(),
        }
    }
}
