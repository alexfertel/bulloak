//! Defines a high-level intermediate representation (HIR) and a translate fn
//! that takes a tree and returns its corresponding HIR.

pub mod combiner;
#[allow(clippy::module_inception)]
pub mod hir;
pub mod translator;
pub mod visitor;

pub use hir::*;

use crate::{
    config::Config, error, scaffold::modifiers::ModifierDiscoverer, syntax,
    utils::split_trees,
};

/// High-level function that returns a HIR given the contents of a `.tree` file.
///
/// This function leverages [`translate_tree_to_hir`] to generate the HIR for
/// each tree, and [`crate::hir::combiner::Combiner::combine`] to combine the
/// HIRs into a single HIR.
pub fn translate(text: &str, cfg: &Config) -> anyhow::Result<Hir> {
    Ok(translate_and_combine_trees(text, cfg)?)
}

/// High-level function that returns a HIR given the contents of a `.tree` file.
///
/// This function leverages [`translate_tree_to_hir`] to generate the HIR for
/// each tree, and [`crate::hir::combiner::Combiner::combine`] to combine the
/// HIRs into a single HIR.
pub(crate) fn translate_and_combine_trees(
    text: &str,
    cfg: &Config,
) -> error::Result<Hir> {
    let trees = split_trees(text);
    let hirs = trees
        .map(|tree| translate_tree_to_hir(tree, cfg))
        .collect::<error::Result<Vec<Hir>>>()?;
    Ok(combiner::Combiner::new().combine(text, hirs)?)
}

/// Generates the HIR for a single tree.
///
/// This function leverages [`crate::syntax::parse`] and
/// [`crate::hir::translator::Translator::translate`] to hide away most of the
/// complexity of `bulloak`'s internal compiler.
pub fn translate_tree_to_hir(tree: &str, cfg: &Config) -> error::Result<Hir> {
    let ast = syntax::parse(tree)?;
    let mut discoverer = ModifierDiscoverer::new();
    let modifiers = discoverer.discover(&ast);
    Ok(translator::Translator::new().translate(&ast, modifiers, cfg))
}
