# Bulloak Contribution Guidelines

Thank you for contributing to this project! We expect all contributors to have read this file before submitting PRs.

## Pull Requests

To make changes to `bulloak`, please submit a PR to the `main` branch.
We'll review them and either merge or request changes.
We have a basic CI setup that runs:

```rust
cargo check
cargo test
cargo fmt --all -- --check
```

## Testing

PRs without tests when appropriate won't be merged.

To run the tests:

```bash
cargo test
```

## Formatting

We adhere to the standard rules of formatting in rust.
Please, make sure that your changes follow them too by running:

```bash
cargo fmt
```

