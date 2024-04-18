//! Defines a high-level intermediate representation (HIR) and a translate fn
//! that takes a tree and returns its corresponding HIR.

pub mod combiner;
pub mod hir;
pub mod translator;
pub mod visitor;

pub use hir::*;

/// High-level function that returns a HIR given the contents of a `.tree` file.
///
/// This function leverages `translate_tree_to_hir` to generate the HIR for each tree,
/// and `crate::hir::combiner::Combiner::combine` to combine the HIRs into a single HIR.
pub fn translate(text: &str, add_vm_skip: bool) -> anyhow::Result<Hir> {
    Ok(translate_and_combine_trees(text, add_vm_skip)?)
}

/// Generates the HIR for a single tree.
///
/// This function leverages `crate::syntax::parse` and `crate::hir::translator::Translator::translate`
/// to hide away most of the complexity of `bulloak`'s internal compiler.
pub fn translate_tree_to_hir(
    tree: &str,
    add_vm_skip: bool,
) -> crate::error::Result<crate::hir::Hir> {
    let ast = crate::syntax::parse(tree)?;
    let mut discoverer = crate::scaffold::modifiers::ModifierDiscoverer::new();
    let modifiers = discoverer.discover(&ast);
    Ok(crate::hir::translator::Translator::new().translate(&ast, modifiers, add_vm_skip))
}

/// High-level function that returns a HIR given the contents of a `.tree` file.
///
/// This function leverages `translate_tree_to_hir` to generate the HIR for each tree,
/// and `crate::hir::combiner::Combiner::combine` to combine the HIRs into a single HIR.
pub(crate) fn translate_and_combine_trees(
    text: &str,
    add_vm_skip: bool,
) -> crate::error::Result<crate::hir::Hir> {
    let trees = crate::utils::split_trees(text);
    let hirs = trees
        .map(|tree| translate_tree_to_hir(tree, add_vm_skip))
        .collect::<crate::error::Result<Vec<crate::hir::Hir>>>()?;
    Ok(crate::hir::combiner::Combiner::new().combine(text, hirs)?)
}
