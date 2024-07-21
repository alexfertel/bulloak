//! The syntax parser module.
//!
//! This module includes everything necessary to convert from a tree
//! in string form to an AST. It also includes a semantic analyzer.

mod ast;
mod error;
pub mod parser;
pub mod semantics;
mod span;
mod splitter;
mod test_utils;
pub mod tokenizer;
pub mod utils;
mod visitor;

pub use ast::{Action, Ast, Condition, Description, Root};
pub use error::FrontendError;
pub use span::{Position, Span};
pub use tokenizer::{Token, TokenKind};
pub use visitor::Visitor;

/// Parses a string containing trees into ASTs.
pub fn parse(text: &str) -> anyhow::Result<Vec<ast::Ast>> {
    splitter::split_trees(text).map(parse_one).collect()
}

/// Parses a string containing a single tree into an AST.
pub fn parse_one(text: &str) -> anyhow::Result<ast::Ast> {
    let tokens = tokenizer::Tokenizer::new().tokenize(text)?;
    let ast = parser::Parser::new().parse(text, &tokens)?;
    let mut analyzer = semantics::SemanticAnalyzer::new(text);
    analyzer.analyze(&ast)?;

    Ok(ast)
}
