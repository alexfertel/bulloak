//! The syntax parser module.
//!
//! This module includes everything necessary to convert from a tree
//! in string form to an AST. It also includes a semantic analyzer.

pub mod ast;
pub mod error;
pub mod parser;
pub mod semantics;
pub mod span;
pub mod splitter;
mod test_utils;
pub mod tokenizer;
pub mod utils;
pub mod visitor;

/// Parses a string containing trees into ASTs.
pub fn parse(text: &str) -> anyhow::Result<Vec<ast::Ast>> {
    splitter::split_trees(text).map(parse_one).collect()
}

/// Parses a string containing a single tree into an AST.
fn parse_one(text: &str) -> anyhow::Result<ast::Ast> {
    let tokens = tokenizer::Tokenizer::new().tokenize(text)?;
    let ast = parser::Parser::new().parse(text, &tokens)?;
    let mut analyzer = semantics::SemanticAnalyzer::new(text);
    analyzer.analyze(&ast)?;

    Ok(ast)
}
