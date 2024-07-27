# bulloak-foundry

`bulloak-foundry` is a Rust crate that serves as a backend for generating Foundry tests from `bulloak-syntax` Abstract Syntax Trees (ASTs). It provides functionality to scaffold Solidity test files and check existing tests against specifications.

## Features

- Generate `.t.sol` files with scaffolded Foundry tests from `bulloak-syntax` ASTs.
- Check existing Solidity test files against `.tree` specifications.
- Implement and enforce custom rules for test structure and content.
- Automatic fixing of certain rule violations.

## Usage

To use bulloak-foundry in your project, add it to your `Cargo.toml`:

```toml
[dependencies]
bulloak-foundry = "0.1.0"  # Replace with the actual version
```

### Scaffolding Tests

```rust
use bulloak_foundry::scaffold;

fn main() -> anyhow::Result<()> {
    let tree_spec = "Your .tree specification here";
    let foundry_test = scaffold::scaffold(tree_spec)?;

    // Write foundry_test to a .t.sol file

    Ok(())
}
```

## Violation Checking

`bulloak-foundry` includes a system for defining and checking rules against Solidity test files. Violations can be of different kinds, as defined in the `ViolationKind` enum.

## License

This project is licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  https://www.apache.org/licenses/LICENSE-2.0).
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  https://opensource.org/licenses/MIT).
