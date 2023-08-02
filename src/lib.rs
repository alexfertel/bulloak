use std::{fs, io::Result};

mod ast;
mod emitter;
mod error;
mod parser;
mod semantics;
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

    match ast {
        ast::Ast::Root(root) => {
            let mut analyzer = semantics::SemanticAnalyzer::new(&text);
            let errors = analyzer.analyze(&root)?;
            println!("errors: {:#?}", errors);
        }
        _ => unreachable!(),
    }

    Ok(())
}