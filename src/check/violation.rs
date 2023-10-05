//! Defines a rule-checking error object.

use std::fmt;

use crate::utils::repeat_str;

use super::location::Location;

/// An error that occurred while checking specification rules between
/// a tree and a Solidity contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Violation {
    /// The kind of violation.
    kind: ViolationKind,
    location: Location,
}

impl Violation {
    /// Create a new violation.
    pub(crate) fn new(kind: ViolationKind, location: Location) -> Self {
        Self { kind, location }
    }
}

/// The type of an error that occurred while checking specification rules between
/// a tree and a Solidity contract.
///
/// NOTE: Adding a variant to this enum most certainly will mean adding a variant to the
/// `Rules` section of `bulloak`'s README. Please, do not forget to add it if you are
/// implementing a rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ViolationKind {
    /// The corresponding Solidity file does not exist.
    FileMissing,
    /// Couldn't read the corresponding Solidity file.
    FileUnreadable,
    /// Found no matching Solidity contract.
    ContractMissing(String),
    /// Contract name doesn't match.
    ContractNameNotMatches(String, String),
    /// Found a tree element without its matching codegen.
    MatchingCodegenMissing(String),
    /// Found an incorrectly ordered element.
    CodegenOrderMismatch(String, usize),
    /// This enum may grow additional variants, so this makes sure clients
    /// don't count on exhaustive matching. (Otherwise, adding a new variant
    /// could break existing code.)
    #[doc(hidden)]
    __Nonexhaustive,
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let divider = repeat_str("•", 79);
        writeln!(f, "{divider}")?;

        writeln!(f, "check failed: {}", self.kind)?;
        writeln!(f, "{}", self.location)?;

        Ok(())
    }
}

impl fmt::Display for ViolationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::ViolationKind::{
            CodegenOrderMismatch, ContractMissing, ContractNameNotMatches, FileMissing,
            FileUnreadable, MatchingCodegenMissing,
        };
        match self {
            FileMissing => write!(f, r#"the file is missing its matching Solidity file"#,),
            FileUnreadable => {
                write!(f, "bulloak couldn't read the file")
            }
            ContractMissing(contract) => write!(
                f,
                r#"couldn't find a corresponding contract for "{contract}" in the Solidity file"#
            ),
            MatchingCodegenMissing(codegen_name) => write!(
                f,
                r#"couldn't find a corresponding element for "{codegen_name}" in the Solidity file"#
            ),
            ContractNameNotMatches(tree_name, sol_name) => write!(
                f,
                r#"couldn't find a corresponding contract for "{tree_name}" in the Solidity file. Found "{sol_name}""#
            ),
            CodegenOrderMismatch(name_in_tree, line_in_tree) => write!(
                f,
                r#"found a matching element for "{name_in_tree}" in line {line_in_tree}, but the order is not correct"#
            ),
            _ => unreachable!(),
        }
    }
}
