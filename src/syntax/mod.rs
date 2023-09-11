pub mod ast;
pub mod parser;
pub mod semantics;
pub mod tokenizer;

pub fn parse(text: &str) -> crate::error::Result<ast::Ast> {
    let tokens = tokenizer::Tokenizer::new().tokenize(text)?;
    let ast = parser::Parser::new().parse(text, &tokens)?;
    let mut analyzer = semantics::SemanticAnalyzer::new(text);
    analyzer.analyze(&ast)?;

    Ok(ast)
}
