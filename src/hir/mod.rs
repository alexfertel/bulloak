//! Defines a high-level intermediate representation (HIR) and a translate fn
//! that takes a tree and returns its corresponding HIR.

pub mod combiner;
pub mod hir;
pub mod translator;
pub mod visitor;

pub use hir::*;

use super::utils::translate_and_combine_trees;

/// High-level function that returns a HIR given the contents of a `.tree` file.
/// 
/// This function leverages translate_tree_to_hir to generate the HIR for each tree,
/// and crate::hir::combiner::Combiner::combine to combine the HIRs into a single HIR.
pub fn translate(text: &str) -> anyhow::Result<Hir> {
    Ok(translate_and_combine_trees(text)?)
}
