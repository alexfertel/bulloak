# Bulloak Contribution Guidelines

Thank you for contributing to this project! We expect all contributors to have
read this file before submitting PRs.

## Pre-Requisites

Ensure you have the following software installed and configured on your machine:

- [Node.js](https://nodejs.org) (v20+)
- [Rust (Nightly)](https://rust-lang.org/tools/install)

In addition, familiarity with [Solidity](https://soliditylang.org) is requisite.

## Pull Requests

To make changes to `bulloak`, please submit a PR to the `main` branch. We'll
review them and either merge or request changes. We have a basic CI setup that
runs:

```rust
cargo check
cargo test
rustup run nightly cargo fmt --all -- --check
```

## Testing

PRs without tests when appropriate won't be merged.

To run the tests:

```bash
cargo test
```

## Formatting

### Rust Code

We adhere to the standard rules of formatting in rust. Please, make sure that
your changes follow them too by running:

```bash
rustup run nightly cargo fmt
```

### Markdown Files

We use Prettier to enforce consistent formatting across Markdown files.

```bash
# Check formatting
npm run prettier:check

# Fix formatting issues
npm run prettier:write
```
