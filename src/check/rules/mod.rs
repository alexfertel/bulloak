//! Defines rules that Solidity contracts must follow in order to
//! be considered spec compliant.
//!
//! These rules are checked with the `bulloak check` command.

use solang_parser::pt::SourceUnit;

use crate::hir::Hir;

use super::violation::Violation;

pub(crate) mod structural_match;

/// The context in which rule-checking happens.
///
/// This is a utility struct that abstracts away the requirements
/// for a `check` call. If you need any additional information
/// for your rule, feel free to add it here.
pub(crate) struct Context<'c> {
    /// The path to the tree file.
    pub(crate) tree_path: &'c str,
    /// The high-level intermediate representation
    /// of the bulloak tree.
    pub(crate) tree_hir: &'c Hir,
    /// The path to the Solidity file.
    pub(crate) sol_path: &'c str,
    /// The contents of the Solidity file.
    pub(crate) sol_contents: &'c str,
    /// The abstract syntax tree of the Solidity file.
    pub(crate) sol_ast: &'c SourceUnit,
}

/// Trait definition for a rule checker object.
///
/// All children modules must export an implementor of this trait.
pub(crate) trait Checker {
    fn check(ctx: &Context) -> anyhow::Result<Vec<Violation>>;
}
