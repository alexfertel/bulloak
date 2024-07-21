//! Defines a high-level intermediate representation (HIR) and translation
//! functions that convert abstract syntax trees (ASTs) to their corresponding
//! HIR.

pub mod combiner;
#[allow(clippy::module_inception)]
pub mod hir;
pub use hir::*;
pub mod translator;
pub mod visitor;

use bulloak_syntax::Ast;

use crate::{config::Config, scaffold::modifiers::ModifierDiscoverer};

/// Translates the contents of a `.tree` file into a HIR.
///
/// # Arguments
///
/// * `text` - The contents of the `.tree` file.
/// * `cfg` - The configuration for the translation process.
///
/// # Returns
///
/// Returns a `Result` containing the translated `Hir` or a `TranslationError`.
pub fn translate(text: &str, cfg: &Config) -> anyhow::Result<Hir> {
    let asts = bulloak_syntax::parse(text)?;

    if asts.len() == 1 {
        return Ok(ast_to_hir(&asts[0], cfg));
    }

    let hirs = asts.into_iter().map(|ast| ast_to_hir(&ast, cfg));
    Ok(combiner::Combiner::new().combine(text, hirs)?)
}

/// Generates the HIR for a single AST.
///
/// # Arguments
///
/// * `ast` - The Abstract Syntax Tree to translate.
/// * `cfg` - The configuration for the translation process.
///
/// # Returns
///
/// Returns the translated `Hir`.
pub fn ast_to_hir(ast: &Ast, cfg: &Config) -> Hir {
    let mut discoverer = ModifierDiscoverer::new();
    let modifiers = discoverer.discover(&ast);
    translator::Translator::new().translate(ast, modifiers, cfg)
}
