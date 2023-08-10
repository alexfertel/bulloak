<p align="center">
    <img src="https://github.com/alexfertel/bulloak/assets/22298999/adffdf7f-ae18-4db5-a276-449852c8dd0a" width="300"></a>
    <a href="https://crates.io/crates/bulloak/">
        <img src="https://img.shields.io/crates/v/bulloak?style=flat&labelColor=1C2C2E&color=C96329&logo=Rust&logoColor=white">
    </a>
</p>

# bulloak

A simple, fast, and easy to use Solidity test generator based on the
[Branching Tree Technique](https://twitter.com/PaulRBerg/status/1679914755014942720?s=20).

## Installing

```bash
cargo install bulloak
```

## Usage

### Basic Usage

Say you have a `foo.tree` file with the following contents:

```text
foo.sol
 └── when stuff called
    └── it should revert
```

If you pass it to `bulloak` like so, you will get the skeleton
of a test contract printed to `stdout`:

```
$ bulloak foo.tree
pragma solidity [VERSION];

contract FooTest {
  modifier whenStuffCalled() {
    _;
  }

  function testRevertsWhenStuffCalled()
    external
    whenStuffCalled
  {
    // it should revert
  }
}
```

### Scaffold Multiple Trees

If you are working in a solidity project and you have
multiple trees you want to scaffold, you can use the `-w` option.

```
$ bulloak -w ./**/*.tree
```

This will create `solidity` files with the same name as the `.tree`
files with the result of scaffolding each tree.

### Options

```
Usage: bulloak [OPTIONS] [FILES]...

Arguments:
  [FILES]...  .tree files to process

Options:
  -c                 Whether to print `it` branches as comments in the output code
  -i <INDENT>        The indentation of the output code [default: 2]
  -w, --write-files  Whether to write to files instead of stdout
  -h, --help         Print help (see more with '--help')
  -V, --version      Print version
```

### Compiler Errors

Another feature of `bulloak` is reporting errors in your input trees.

For example, say you have a buggy `foo.tree` file, which is missing a
`└` character. Running `bulloak foo.tree` would report the error like this:

```
•••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••
bulloak error: unexpected `when` keyword

── when the id references a null stream
   ^^^^

--- (line 2, column 4) ---
file: foo.tree
```

## Contributing

Please refer to [CONTRIBUTING.md](./CONTRIBUTING.md).

## Inspired By

`bulloak` is heavily inspired by [BurntSushi's regex crate](https://github.com/rust-lang/regex).

## License

This project is licensed under either of:

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0).
* MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT).

