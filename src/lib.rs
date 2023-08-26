/*! # Bulloak

This crade provides a library and a binary for generating Solidity code from
`.tree` files.

## Installation

To install the binary, run:

```bash
cargo install bulloak
```

## Usage

At the moment, this crate is mainly meant to be used as a binary, however,
this crate is [on crates.io](https://crates.io/crates/bulloak) and can be
used by adding `bulloak` to your dependencies in your project's `Cargo.toml`.

```toml
[dependencies]
bulloak = "0.4.0"
```

Then, you would use it like so:

```rust
use bulloak::Scaffolder;

let text = String::from(
"foo.sol
 └── when stuff called
    └── it should revert");

let scaffolder = Scaffolder::new(false, 4, "0.8.0");

assert_eq!(
    scaffolder.scaffold(&text).unwrap().emitted,
    r"pragma solidity 0.8.0;

contract FooTest {
    modifier whenStuffCalled() {
        _;
    }

    function test_RevertWhen_StuffCalled()
        external
        whenStuffCalled
    {
    }
}");
```

## Implementation

The main purpose of the crate is to generate a basic skeleton of a
test suite described in a `.tree` file using the Branching Tree Technique.
We chose to implement a compiler for this purpose, as it gives us a
lot of flexibility in the future.

### Syntax

There is no formal definition of the syntax supported by this compiler yet.
However, the following is a rough (and simplified) example of what it looks like:

```text
{CONTRACT NAME}.sol
[├ any number of conditions or actions]
 └── [when | given] {CONDITION TITLE}
   [├ any number of conditions or actions]
    └── it {ACTION TITLE}
```

Note that this is a tree-like structure, hence the name of the technique.

### Example

Here is an example of a `.tree` file:

```text
foo.sol
└── when stuff called
   └── it should revert
```

This will generate the following Solidity code:

```solidity
pragma solidity 0.8.0;

contract FooTest {
    modifier whenStuffCalled() {
        _;
    }

    function test_RevertWhen_StuffCalled()
        external
        whenStuffCalled
    {
    }
}
```

Note that we follow Foundry's naming practices for tests.

### Reverts

Note that the special action `it should revert` will generate a test with
`Revert` in the name.

### Comments

If the `--with-actions-as-comments` flag (also `-c`) is passed, the compiler
will generate comments for each action in the tree. For example, the following
`.tree` file:

```text
foo.sol
└── when stuff called
   ├── it should setup something
   └── it should do something else
```

Will generate the following Solidity code:

```solidity
pragma solidity 0.8.0;

contract FooTest {
    modifier whenStuffCalled() {
        _;
    }

    function testWhenStuffCalled()
        external
        whenStuffCalled
    {
        // it should setup something
        // it should do something else
    }
}
```
*/

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

/// Utility struct that holds any useful information resulting
/// from the compilation of a `.tree` file.
///
/// This will be populated by the `scaffold` function.
pub struct Scaffolded {
    /// The emitted Solidity code.
    pub emitted: String,
    /// The name of the output file.
    ///
    /// This is _exactly_ the filename at the top of the `.tree` file.
    pub output_file: String,
}

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
    pub fn scaffold(&self, text: &str) -> error::Result<Scaffolded> {
        let tokens = tokenizer::Tokenizer::new().tokenize(text)?;
        let ast = parser::Parser::new().parse(text, &tokens)?;
        let mut analyzer = semantics::SemanticAnalyzer::new(text);
        analyzer.analyze(&ast)?;
        let mut discoverer = modifiers::ModifierDiscoverer::new();
        let modifiers = discoverer.discover(&ast);
        let emitted = emitter::Emitter::new(self.with_comments, self.indent, self.solidity_version)
            .emit(&ast, modifiers);

        let output_file = match ast {
            ast::Ast::Root(root) => root.file_name,
            // It's impossible to get here, as the parser will always return
            // an `Ast::Root` variant.
            _ => unreachable!(),
        };

        Ok(Scaffolded {
            emitted,
            output_file,
        })
    }
}
