use std::{fs, io::Result};

mod ast;
mod error;
mod parser;
mod span;
mod tokenizer;
mod visitor;

pub fn run(file_name: &str) -> Result<()> {
    let text = fs::read_to_string(file_name)?;

    if let Err(err) = scaffold(&text) {
        eprintln!("{}", err);
        std::process::exit(1);
    }

    Ok(())
}

fn scaffold(text: &str) -> error::Result<()> {
    let tokens = tokenizer::Tokenizer::new().tokenize(&text)?;
    println!("Tokens: {:#?}", tokens);

    let ast = parser::Parser::new().parse(&text, &tokens)?;
    println!("AST: {:#?}", ast);

    Ok(())
}
