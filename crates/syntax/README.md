# bulloak-syntax

## Overview

`bulloak-syntax` is a Rust crate that provides a syntax parser for converting tree-like structures in string form into
Abstract Syntax Trees (ASTs). It also includes a semantic analyzer for further processing of the parsed structures.

## Features

- Parse strings containing tree-like structures into ASTs.
- Tokenize input strings.
- Perform semantic analysis on parsed ASTs (e.g., ensure the tree has content, top‑level actions are unique). Duplicate
  condition titles are allowed; only duplicate top‑level actions are rejected.
- Support for parsing both single and multiple trees.
- Error handling with custom `FrontendError` type.

## Usage

To use bulloak-syntax in your project, add it to your `Cargo.toml`:

```toml
[dependencies]
bulloak-syntax = "0.1.0"  # Replace with the actual version
```

And then parse the input:

```rust
use bulloak_syntax::parse;

fn main() -> anyhow::Result<()> {
    let input = "your tree-like structure here";
    let asts = parse(input)?;

    // Process the ASTs as needed
    for ast in asts {
        // ...
    }

    Ok(())
}
```

## License

This project is licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0).
- MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT).
