//! Defines rules that solidity contracts must follow in order to
//! be considered spec compliant.

use solang_parser::pt::SourceUnit;

use crate::hir::Hir;

use super::violation::Violation;

pub(crate) mod structural_match;

/// The context in which rule-checking happens.
///
/// This is a utility struct that abstracts away the requirements
/// for a `check` call. If you need any additional information
/// for your rule, feel free to add it here.
pub(crate) struct Context<'h> {
    /// The path to the tree file.
    pub(crate) tree_path: &'h str,
    /// The high-level intermediate representation
    /// of the bulloak tree.
    pub(crate) tree_hir: &'h Hir,
    /// The path to the solidity file.
    pub(crate) sol_path: &'h str,
    /// The abstract syntax tree of the solidity file.
    pub(crate) sol_ast: &'h SourceUnit,
}

/// Trait definition for a rule checker object.
///
/// All children modules must export an implementor of this trait.
pub(crate) trait Checker {
    fn check(ctx: &Context) -> anyhow::Result<Vec<Violation>>;
}
