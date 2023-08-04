use clap::Parser;
use std::{fs, io::Result, path::PathBuf};

mod ast;
mod emitter;
mod error;
mod modifiers;
mod parser;
mod semantics;
mod span;
mod tokenizer;
mod utils;
mod visitor;

#[derive(Parser)]
#[command(author, version, about, long_about = None)] // Read from `Cargo.toml`
pub struct Config {
    /// .tree files to process.
    files: Vec<PathBuf>,

    /// Whether to print `it` branches as comments
    /// in the output code.
    #[arg(short = 'c', default_value = "true")]
    with_actions_as_comments: bool,

    /// The indentation of the output code.
    #[arg(short = 'i', default_value = "2")]
    indent: usize,
}

pub fn run(config: &Config) -> Result<()> {
    for file in config.files.iter() {
        let text = fs::read_to_string(file)?;
        match scaffold(&text, &config) {
            Ok(code) => {
                println!("{}", code);
            }
            Err(err) => {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        };
    }

    Ok(())
}

fn scaffold(text: &str, config: &Config) -> error::Result<String> {
    let tokens = tokenizer::Tokenizer::new().tokenize(&text)?;
    let ast = parser::Parser::new().parse(&text, &tokens)?;
    match ast {
        ast::Ast::Root(ref root) => {
            let mut analyzer = semantics::SemanticAnalyzer::new(&text);
            analyzer.analyze(&root)?;
        }
        _ => unreachable!(),
    }
    let mut discoverer = modifiers::ModifierDiscoverer::new();
    let modifiers = discoverer.discover(&ast);
    let solcode = emitter::Emitter::new(config.with_actions_as_comments, config.indent)
        .emit(&ast, &modifiers);

    Ok(solcode)
}
