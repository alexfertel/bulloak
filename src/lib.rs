use std::fs;

use crate::tokenizer::Tokenizer;

mod span;
mod tokenizer;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn run(file_name: &str) -> Result<()> {
    let tree = fs::read_to_string(file_name)?;

    println!("Tree: {}", tree);

    let tokens = Tokenizer::new().tokenize(&tree)?;
    println!("Tokens: {:#?}", tokens);

    Ok(())
}
