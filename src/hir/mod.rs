//! Defines a high-level intermediate representation (HIR) and a translate fn
//! that takes a tree and returns its corresponding HIR.

pub mod hir;
pub mod translator;
pub mod visitor;

pub use hir::*;

/// High-level function that returns a HIR given the contents of a `.tree` file.
///
/// This function leverages `crate::syntax::parse` and `translator::Translator::translate`
/// to hide away most of the complexity of `bulloak`'s internal compiler.
pub fn translate(tree: &str) -> anyhow::Result<Hir> {
    let ast = crate::syntax::parse(&tree)?;
    let mut discoverer = crate::scaffold::modifiers::ModifierDiscoverer::new();
    let modifiers = discoverer.discover(&ast);
    Ok(translator::Translator::new().translate(&ast, modifiers))
}
