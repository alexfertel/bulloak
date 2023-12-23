//! Defines a high-level intermediate representation (HIR) and a translate fn
//! that takes a tree and returns its corresponding HIR.

pub mod combiner;
pub mod hir;
pub mod translator;
pub mod visitor;

pub use hir::*;

use super::utils::split_trees;

/// High-level function that returns a HIR given the contents of a `.tree` file.
///
/// This function leverages `crate::syntax::parse` and `translator::Translator::translate`
/// to hide away most of the complexity of `bulloak`'s internal compiler.
pub fn translate(text: &str) -> anyhow::Result<Hir> {
    let combiner = combiner::Combiner::new();
    let trees = split_trees(text);

    let mut discoverer = crate::scaffold::modifiers::ModifierDiscoverer::new();
    let translator = translator::Translator::new();

    let mut errors = Vec::new();
    let asts: Vec<crate::syntax::ast::Ast> = trees
        .iter()
        .filter_map(|&tree| {
            crate::syntax::parse(tree)
                .map_err(|error| errors.push(error))
                .ok()
        }) // @follow-up - how do we combine and report errors?
        .collect();

    let hirs: Vec<hir::Hir> = asts
        .iter()
        .map(|ast| {
            let modifiers = discoverer.discover(ast);
            translator.translate(ast, modifiers)
        })
        .collect();

    combiner.verify(&asts).unwrap(); // @follow-up - a lot of this code is duplicated from the `scaffold` module, how best do we DRY it up?

    Ok(combiner.combine(&hirs))
}
