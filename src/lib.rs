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
bulloak = "0.1.0"
```

Then, you would use it like so:

```rust
# use bulloak::scaffold;
let text = String::from(
"foo.sol
 └── when stuff called
    └── it should revert");

assert_eq!(
    &scaffold(&text, false, 4).unwrap(),
    r"pragma solidity [VERSION];

contract FooTest {
    modifier whenStuffCalled() {
        _;
    }

    function testRevertsWhenStuffCalled()
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
 └── when {CONDITION TITLE}
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
pragma solidity [VERSION];

contract FooTest {
    modifier whenStuffCalled() {
        _;
    }

    function testRevertsWhenStuffCalled()
        external
        whenStuffCalled
    {
    }

}
```

Note that the name of the contract is inferred from the name of the file.
In future versions of the compiler, a better mechanism for this might be
implemented. Also note that `[VERSION]` is a placeholder for the version of
Solidity used in the file. We cannot infer that yet.

### Reverts

Note that the special action `it should revert` will generate a test with
`Reverts` in the name.

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
pragma solidity [VERSION];

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

/// Generates Solidity code from a `.tree` file.
///
/// See the [crate-level documentation] for details.
///
///   [crate-level documentation]: ./index.html
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
