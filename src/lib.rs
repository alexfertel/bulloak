use std::fs;

use crate::tokenizer::Tokenizer;

mod ast;
mod parser;
mod span;
mod tokenizer;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn run(file_name: &str) -> Result<()> {
    let tree = fs::read_to_string(file_name)?;

    println!("Tree: {}", tree);

    let tokens = Tokenizer::new().tokenize(&tree)?;
    println!("Tokens: {:#?}", tokens);

    let ast = parser::Parser::new().parse(&tokens)?;
    println!("AST: {:#?}", ast);

    Ok(())
}
