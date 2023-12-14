//! Defines a high-level intermediate representation (HIR) and a translate fn
//! that takes a tree and returns its corresponding HIR.

pub mod combiner;
pub mod hir;
pub mod translator;
pub mod visitor;

pub use hir::*;

/// High-level function that returns a HIR given the contents of a `.tree` file.
///
/// This function leverages `crate::syntax::parse` and `translator::Translator::translate`
/// to hide away most of the complexity of `bulloak`'s internal compiler.
pub fn translate(tree: &str) -> anyhow::Result<Hir> {
    let asts = crate::syntax::parse(tree)?;
    let combiner = combiner::Combiner::new();
    match combiner.verify(&asts) {
        Err(_e) => {
            // @follow-up - propagate this error
        }
        _ => {}
    }
    let mut hirs = Vec::new();
    let mut discoverer = crate::scaffold::modifiers::ModifierDiscoverer::new();
    let translator = translator::Translator::new();
    for ast in &asts {
        // @follow-up - use `map` instead of `for _ in`, here and elsewhere?
        let modifiers = discoverer.discover(ast).unwrap();
        hirs.push(translator.clone().translate(ast, modifiers));
    }

    Ok(combiner.combine(hirs))
}
