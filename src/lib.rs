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
    scaffolder.scaffold(&text).unwrap(),
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

#[doc(hidden)]
pub mod cli;
mod scaffold;

pub use scaffold::Scaffolder;
