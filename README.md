<p align="center">
    <img src="https://github.com/alexfertel/bulloak/assets/22298999/73969e04-1674-4805-b4a5-ff15582d52fc" width="300"></a>
    <br>
    <a href="https://crates.io/crates/bulloak/">
        <img src="https://img.shields.io/crates/v/bulloak?style=flat&labelColor=1C2C2E&color=C96329&logo=Rust&logoColor=white">
    </a>
    <a href="https://codecov.io/gh/alexfertel/bulloak">
        <img src="https://codecov.io/github/alexfertel/bulloak/coverage.svg?branch=main">
    </a>
</p>

# bulloak

A simple, fast, and easy-to-use Solidity test generator based on the
[Branching Tree Technique](https://twitter.com/PaulRBerg/status/1682346315806539776).

- [Installation](#installation)
- [Usage](#usage)
  - [Scaffold Multiple Trees](#scaffold-multiple-trees)
  - [CLI Options](#cli-options)
  - [Compiler Errors](#compiler-errors)
- [Trees](#trees)
  - [Terminology](#terminology)
  - [Spec](#spec)
- [Output](#output)
- [Contributing](#contributing)
- [License](#license)

## Installation

```bash
cargo install bulloak
```

## Usage

Say you have a `foo.tree` file with the following contents:

```text
FooTest
 └── When stuff called
    └── It should revert.
```

If you pass it to `bulloak` like so, you will get the skeleton
of a test contract printed to `stdout`:

```text
$ bulloak foo.tree
pragma solidity 0.8.0;

contract FooTest {
  modifier whenStuffCalled() {
    _;
  }

  function test_RevertWhen_StuffCalled()
    external
    whenStuffCalled
  {
    // It should revert.
  }
}
```

### Scaffold Multiple Trees

If you are working in a solidity project and you have
multiple trees you want to scaffold, you can use the `-w` option.

```text
$ bulloak -w ./**/*.tree
```

This will create `solidity` files with the same name as the `.tree`
files with the result of scaffolding each tree.

If there exists a file with a title that matches the name at the
root node of the `.tree`, then `bulloak` will skip writing that file.
However, you may override this behaviour with the `-f` flag. This
will write to the file system overwriting any files that exist.

```text
$ bulloak -wf ./**/*.tree
```

### Compiler Errors

Another feature of `bulloak` is reporting errors in your input trees.

For example, say you have a buggy `foo.tree` file, which is missing a
`└` character. Running `bulloak foo.tree` would report the error like this:

```text
•••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••
bulloak error: unexpected `when` keyword

── when the id references a null stream
   ^^^^

--- (line 2, column 4) ---
file: foo.tree
```

### CLI Options

```text
Usage: bulloak [OPTIONS] [FILES]...

Arguments:
  [FILES]...  .tree files to process

Options:
  -c
          Whether to print `it` branches as comments in the output code
  -i <INDENT>
          The indentation of the output code [default: 2]
  -w, --write-files
          Whether to write to files instead of stdout
  -f, --force-write
          When `write_files` is specified, use `--force-write` to overwrite the output files
  -s, --solidity-version <SOLIDITY_VERSION>
          Sets a solidity version for the test contracts [default: 0.8.0]
  -h, --help
          Print help (see more with '--help')
  -V, --version
          Print version
```

## Trees

`bulloak` scaffolds solidity test files based on `.tree` specifications
that follow the [Branching Tree Technique](https://twitter.com/PaulRBerg/status/1682346315806539776).

Currently, there is on-going [discussion](https://github.com/alexfertel/bulloak/discussions) on how to handle different edge-cases to better empower the solidity community. This section is a description of the current implementation of the compiler.

### Terminology

- *Condition*: `when/given` branches of a tree.
- *Action*: `it` branches of a tree. Every action is a leaf node of the tree.

### Spec

Each `tree` file should describe a function under test. Trees follow these rules:

- The line at the top of the file is the name of the contract.
- `bulloak` expects you to use `├` and `└` characters to denote branches.
- Every branch *must* start with one of `when`, `given` or `it`.
- If a branch starts with either `when` or `given`, it is a condition.
- `when` and `given` are interchangeable.
- If a branch starts with `it`, it is an action.
- Keywords are case-insensitive: `it` is the same as `It` and `IT`.

Take the following solidity function:

```solidity
function hashPair(bytes32 a, bytes32 b) private pure returns (bytes32) {
    return a < b ? hash(a, b) : hash(b, a);
}
```
A reasonable spec for the above function would be:
```text
HashPairTest
├── It should never revert.
├── When first arg is smaller than second arg
│   └── It should match the result of `keccak256(abi.encodePacked(a,b))`.
└── When first arg is bigger than second arg
    └── It should match the result of `keccak256(abi.encodePacked(b,a))`.
```

There is a top-level action which would generate a test to check the function invariant that it should never revert.

Then, we have the two possible preconditions: `a < b` and `a >= b`. Both branches end in an action that will make `bulloak` generate the respective test.

Note the following things:

- Actions are written with ending dots but conditions are not. This is because actions support any character, but conditions don't. Since conditions are transformed into modifiers, they have to be valid solidity identifiers.
- You can have top-level actions without conditions. Currently, `bulloak` also supports actions with sibling conditions, but this might get removed in a future version per this [discussion](https://github.com/alexfertel/bulloak/issues/22).
- The root of the tree will be emitted as the name of the test contract.

## Output

There are a few things to keep in mind about the scaffolded solidity test:

- The contract filename is the same as the `.tree` but with a `.t.sol` extension. E.g. `test.tree` would correspond to `test.t.sol`.
- Test are emitted in the order their corresponding actions appear in the `.tree` file.
- Currently, we generate one modifier per condition, but this might change per this [discussion](https://github.com/alexfertel/bulloak/discussions/7).
- Test names follow [Foundry's best practices](https://book.getfoundry.sh/tutorials/best-practices?highlight=best#tests).


## Contributing

Please refer to [CONTRIBUTING.md](./CONTRIBUTING.md).

## License

This project is licensed under either of:

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0).
* MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT).

