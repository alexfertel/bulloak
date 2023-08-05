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

pub fn scaffold(text: &str, with_comments: bool, indent: usize) -> error::Result<String> {
    let tokens = tokenizer::Tokenizer::new().tokenize(text)?;
    let ast = parser::Parser::new().parse(text, &tokens)?;
    let mut analyzer = semantics::SemanticAnalyzer::new(text);
    analyzer.analyze(&ast)?;
    let mut discoverer = modifiers::ModifierDiscoverer::new();
    let modifiers = discoverer.discover(&ast);
    let solcode = emitter::Emitter::new(with_comments, indent).emit(&ast, modifiers);

    Ok(solcode)
}
