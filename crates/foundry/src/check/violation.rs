//! Defines a rule-checking error object.
use std::{borrow::Cow, fmt};

use bulloak_syntax::FrontendError;
use forge_fmt::solang_ext::SafeUnwrap;
use owo_colors::OwoColorize;
use solang_parser::pt;
use thiserror::Error;

use super::{context::Context, location::Location};
use crate::hir;

/// An error that occurred while checking specification rules between
/// a tree and a Solidity contract.
#[derive(Debug, Error, PartialEq)]
pub struct Violation {
    /// The kind of violation.
    #[source]
    pub kind: ViolationKind,
    /// The location information about this violation.
    pub location: Location,
}

impl Violation {
    /// Create a new violation.
    pub fn new(kind: ViolationKind, location: Location) -> Self {
        Self { kind, location }
    }

    /// Determines whether a given violation is fixable.
    pub fn is_fixable(&self) -> bool {
        self.kind.is_fixable()
    }
}

/// The type of an error that occurred while checking specification rules
/// between a tree and a Solidity contract.
///
/// NOTE: Adding a variant to this enum most certainly will mean adding a
/// variant to the `Rules` section of `bulloak`'s README. Please, do not forget
/// to add it if you are implementing a rule.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ViolationKind {
    /// Found no matching Solidity contract.
    ///
    /// (contract name)
    #[error("contract \"{0}\" is missing in .sol")]
    ContractMissing(String),

    /// Contract name doesn't match.
    ///
    /// (tree name, sol name)
    #[error("contract \"{0}\" is missing in .sol -- found \"{1}\" instead")]
    ContractNameNotMatches(String, String),

    /// The corresponding Solidity file does not exist.
    #[error("the tree is missing its matching Solidity file: {0}")]
    SolidityFileMissing(String),

    /// Couldn't read the corresponding Solidity file.
    #[error("bulloak couldn't read the file")]
    FileUnreadable,

    /// Found an incorrectly ordered element.
    ///
    /// (pt function, current position, insertion position)
    #[error("incorrect position for function `{}`", .0.name.safe_unwrap())]
    FunctionOrderMismatch(pt::FunctionDefinition, usize, usize),

    /// Found a tree element without its matching codegen.
    ///
    /// (hir function, insertion position)
    #[error("function \"{}\" is missing in .sol", .0.identifier.clone())]
    MatchingFunctionMissing(hir::FunctionDefinition, usize),

    /// The parsing of a tree or a Solidity file failed.
    #[error("{}", format_frontend_error(.0))]
    ParsingFailed(#[from] anyhow::Error),
}

impl ViolationKind {
    /// Whether this violation kind is fixable.
    pub fn is_fixable(&self) -> bool {
        matches!(
            self,
            ViolationKind::ContractMissing(_)
                | ViolationKind::ContractNameNotMatches(_, _)
                | ViolationKind::FunctionOrderMismatch(_, _, _)
                | ViolationKind::MatchingFunctionMissing(_, _)
        )
    }

    /// Optionally returns a help text to be used when displaying the violation
    /// kind.
    pub fn help(&self) -> Option<Cow<'static, str>> {
        let text = match self {
            ViolationKind::ContractMissing(name) => {
                format!(r#"consider adding a contract with name "{name}""#)
                    .into()
            }
            ViolationKind::ContractNameNotMatches(name, _) => {
                format!(r#"consider renaming the contract to "{name}""#).into()
            }
            ViolationKind::SolidityFileMissing(filename) => {
                let filename = filename.replace(".t.sol", ".tree");
                format!("consider running `bulloak scaffold {filename}`").into()
            }
            ViolationKind::FunctionOrderMismatch(_, _, _) => {
                "consider reordering the function in the file".into()
            }
            _ => return None,
        };

        Some(text)
    }

    /// Returns a new context with this violation fixed.
    pub fn fix(&self, ctx: Context) -> anyhow::Result<Context> {
        match self {
            ViolationKind::ContractMissing(_) => ctx.fix_contract_missing(),
            ViolationKind::ContractNameNotMatches(new_name, old_name) => {
                ctx.fix_contract_rename(new_name, old_name)
            }
            // Assume order violations have been taken care of first.
            ViolationKind::MatchingFunctionMissing(fn_hir, index) => {
                ctx.fix_matching_fn_missing(fn_hir, *index)
            }
            _ => Ok(ctx),
        }
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}: {}", "warn".yellow(), self.kind)?;
        if let Some(help_text) = self.kind.help() {
            writeln!(f, "     {} help: {}", "=".blue(), help_text)?;
        }
        if self.kind.is_fixable() {
            let file = self.location.file().replace(".t.sol", ".tree");
            write!(f, "     {} fix: run ", "+".blue())?;
            writeln!(f, "`bulloak check --fix {file}`")?;
        }
        writeln!(f, "   {} {}", "-->".blue(), self.location)?;

        Ok(())
    }
}

impl PartialEq for ViolationKind {
    fn eq(&self, other: &Self) -> bool {
        use ViolationKind::*;

        match (self, other) {
            (ContractMissing(a), ContractMissing(b)) => a == b,
            (
                ContractNameNotMatches(a1, a2),
                ContractNameNotMatches(b1, b2),
            ) => a1 == b1 && a2 == b2,
            (SolidityFileMissing(a), SolidityFileMissing(b)) => a == b,
            (FileUnreadable, FileUnreadable) => true,
            (
                FunctionOrderMismatch(f1, cur1, ins1),
                FunctionOrderMismatch(f2, cur2, ins2),
            ) =>
            // Compare on function name and the two positions.
            {
                f1.name == f2.name && cur1 == cur2 && ins1 == ins2
            }
            (
                MatchingFunctionMissing(f1, pos1),
                MatchingFunctionMissing(f2, pos2),
            ) =>
            // Compare on function identifier and the position.
            {
                f1.identifier == f2.identifier && pos1 == pos2
            }
            (ParsingFailed(e1), ParsingFailed(e2)) =>
            // Compare on the formatted error message.
            {
                e1.to_string() == e2.to_string()
            }

            // any mismatched variant
            _ => false,
        }
    }
}

/// Formats frontend errors into human-readable messages.
///
/// # Arguments
/// * `error` - Reference to an `anyhow::Error`
///
/// # Returns
/// A `String` containing the formatted error message
fn format_frontend_error(error: &anyhow::Error) -> String {
    if let Some(error) =
        error.downcast_ref::<bulloak_syntax::tokenizer::Error>()
    {
        format!("an error occurred while parsing the tree: {}", error.kind())
    } else if let Some(error) =
        error.downcast_ref::<bulloak_syntax::parser::Error>()
    {
        format!("an error occurred while parsing the tree: {}", error.kind())
    } else if let Some(error) =
        error.downcast_ref::<crate::hir::combiner::Error>()
    {
        format!("an error occurred while parsing the tree: {}", error.kind())
    } else if error.downcast_ref::<bulloak_syntax::semantics::Error>().is_some()
    {
        "at least one semantic error occurred while parsing the tree".to_owned()
    } else {
        "an error occurred while parsing the solidity file".to_owned()
    }
}
