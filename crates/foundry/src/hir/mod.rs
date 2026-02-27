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

use crate::{
    config::Config, constants::CONTRACT_IDENTIFIER_SEPARATOR,
    scaffold::modifiers::ModifierDiscoverer,
};

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

    for (index, ast) in asts.iter().enumerate() {
        let Ast::Root(root) = ast else {
            continue;
        };
        let separator_count =
            root.contract_name.matches(CONTRACT_IDENTIFIER_SEPARATOR).count();
        if separator_count > 1 {
            anyhow::bail!(
                "an error occurred while parsing the tree: too many separators at tree root #{}. Expected to find at most one `::` between the contract name and the function name",
                index + 1
            );
        }
    }

    if asts.len() == 1 {
        return Ok(translate_one(&asts[0], cfg));
    }

    let hirs = asts.into_iter().map(|ast| translate_one(&ast, cfg));
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
#[must_use]
pub fn translate_one(ast: &Ast, cfg: &Config) -> Hir {
    let mut discoverer = ModifierDiscoverer::new();
    let modifiers = discoverer.discover(ast);
    translator::Translator::new().translate(ast, modifiers, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_rejects_single_root_with_too_many_separators() {
        let tree = "Contract::Function::Extra\n└── It should fail.";
        let result = translate(tree, &Config::default());

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("too many separators at tree root #1"));
    }
}
