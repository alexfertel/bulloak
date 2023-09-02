pub mod ast;
pub mod emitter;
pub mod error;
pub mod modifiers;
pub mod parser;
pub mod semantics;
pub mod span;
pub mod tokenizer;
pub mod utils;
pub mod visitor;

/// The overarching struct that generates Solidity
/// code from a `.tree` file.
pub struct Scaffolder<'s> {
    /// Whether to print `it` branches as comments
    /// in the output code.
    with_comments: bool,
    /// The indentation of the output code.
    indent: usize,
    /// Sets a solidity version for the test contracts.
    solidity_version: &'s str,
}

impl<'s> Scaffolder<'s> {
    /// Creates a new scaffolder with the provided configuration.
    pub fn new(with_comments: bool, indent: usize, solidity_version: &'s str) -> Self {
        Scaffolder {
            with_comments,
            indent,
            solidity_version,
        }
    }
    /// Generates Solidity code from a `.tree` file.
    ///
    /// See the [crate-level documentation] for details.
    ///
    ///   [crate-level documentation]: ./index.html
    pub fn scaffold(&self, text: &str) -> error::Result<String> {
        let tokens = tokenizer::Tokenizer::new().tokenize(text)?;
        let ast = parser::Parser::new().parse(text, &tokens)?;
        let mut analyzer = semantics::SemanticAnalyzer::new(text);
        analyzer.analyze(&ast)?;
        let mut discoverer = modifiers::ModifierDiscoverer::new();
        let modifiers = discoverer.discover(&ast);
        let emitted = emitter::Emitter::new(self.with_comments, self.indent, self.solidity_version)
            .emit(&ast, modifiers);

        Ok(emitted)
    }
}
