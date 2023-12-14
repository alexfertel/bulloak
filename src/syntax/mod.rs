//! The syntax parser module.
//!
//! This module includes everything necessary to convert from a tree
//! in string form to an AST. It also includes a semantic analyzer.

pub mod ast;
pub mod parser;
pub mod semantics;
mod test_utils;
pub mod tokenizer;
pub mod visitor;

/// Parses a tree file into an AST.
pub fn parse(text: &str) -> crate::error::Result<Vec<ast::Ast>> {
    let tokens = tokenizer::Tokenizer::new().tokenize(text)?;
    let asts = parser::Parser::new().parse(text, &tokens)?;
    let mut analyzer = semantics::SemanticAnalyzer::new(text);
    asts.iter().try_for_each(|ast| analyzer.analyze(ast))?;

    Ok(asts)
}
