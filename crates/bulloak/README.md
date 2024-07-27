# bulloak

A Solidity test generator based on the
[Branching Tree Technique](https://twitter.com/PaulRBerg/status/1682346315806539776).

- [Installation](#installation)
  - [VSCode](#vscode)
- [Usage](#usage)
  - [`bulloak scaffold`](#scaffold-solidity-files)
  - [`bulloak check`](#check-that-your-code-and-spec-match)
    - [Rules](#rules)
  - [Compiler Errors](#compiler-errors)
- [Trees](#trees)
  - [Terminology](#terminology)
  - [Spec](#spec)
- [Output](#output)
- [License](#license)

> [!WARNING]
> Note that `bulloak` is still `0.*.*`, so breaking changes
> [may occur at any time](https://semver.org/#spec-item-4). If you must depend
> on `bulloak`, we recommend pinning to a specific version, i.e., `=0.y.z`.

## Installation

```bash
cargo install bulloak
```

### VSCode

The following VSCode extensions are not essential but they are recommended for a
better user experience:

- [Solidity Inspector](https://marketplace.visualstudio.com/items?itemName=PraneshASP.vscode-solidity-inspector) - syntax highlighting for `.tree` files
- [Ascii Tree Generator](https://marketplace.visualstudio.com/items?itemName=aprilandjan.ascii-tree-generator):
  convenient way to generate ASCII trees

## Usage

`bulloak` implements two commands:

- `bulloak scaffold`
- `bulloak check`

### Scaffold Solidity Files

Say you have a `foo.tree` file with the following contents:

```tree
FooTest
└── When stuff is called // Comments are supported.
    └── When a condition is met
        └── It should revert.
            └── Because we shouldn't allow it.
```

You can use `bulloak scaffold` to generate a Solidity contract containing
modifiers and tests that match the spec described in `foo.tree`. The following
will be printed to `stdout`:

```solidity
// $ bulloak scaffold foo.tree
// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;

contract FooTest {
    modifier whenStuffIsCalled() {
        _;
    }

    function test_RevertWhen_AConditionIsMet() external whenStuffIsCalled {
        // It should revert.
        //     Because we shouldn't allow it.
    }
}
```

You can use the `-w` option to write the generated contracts to the file system.
Say we have a bunch of `.tree` files in the current working directory. If we run
the following:

```text
$ bulloak scaffold -w ./**/*.tree
```

`bulloak` will create a `.t.sol` file per `.tree` file and write the generated
contents to it.

If a `.t.sol` file's title matches a `.tree` in the same directory, then
`bulloak` will skip writing to that file. However, you may override this
behaviour with the `-f` flag. This will force `bulloak` to overwrite the
contents of the file.

```text
$ bulloak scaffold -wf ./**/*.tree
```

Note all tests are showing as passing when their body is empty. To prevent this,
you can use the `-S` (or `--vm-skip`) option to add a `vm.skip(true);` at the
beginning of each test function. This option will also add an import for
forge-std's `Test.sol` and all test contracts will inherit from it.

You can skip emitting the modifiers by passing the `-m` (or `--skip--modifiers`)
option. This way, the generated files will only include the test functions.

### Check That Your Code And Spec Match

You can use `bulloak check` to make sure that your Solidity files match your
spec. For example, any missing tests will be reported to you.

Say you have the following spec:

```tree
HashPairTest
├── It should never revert.
├── When first arg is smaller than second arg
│   └── It should match the result of `keccak256(abi.encodePacked(a,b))`.
└── When first arg is bigger than second arg
    └── It should match the result of `keccak256(abi.encodePacked(b,a))`.
```

And a matching Solidity file:

```solidity
pragma solidity 0.8.0;

contract HashPairTest {
  function test_ShouldNeverRevert() external {
    // It should never revert.
  }

  function test_WhenFirstArgIsSmallerThanSecondArg() external {
    // It should match the result of `keccak256(abi.encodePacked(a,b))`.
  }
}
```

This Solidity file is missing the tests for the branch
`When first arg is bigger than second arg`, which would be reported after
running `bulloak check tests/scaffold/basic.tree`, like so:

```text
warn: function "test_WhenFirstArgIsBiggerThanSecondArg" is missing in .sol
     + fix: run `bulloak check --fix tests/scaffold/basic.tree`
   --> tests/scaffold/basic.tree:5

warn: 1 check failed (run `bulloak check --fix <.tree files>` to apply 1 fix)
```

As you can see in the above message, `bulloak` can fix the issue automatically.
If we run the command with the `--stdout` flag, the output is:

```solidity
--> tests/scaffold/basic.t.sol
pragma solidity 0.8.0;

contract HashPairTest {
    function test_ShouldNeverRevert() external {
        // It should never revert.
    }

    function test_WhenFirstArgIsSmallerThanSecondArg() external {
        // It should match the result of `keccak256(abi.encodePacked(a,b))`.
    }

    function test_WhenFirstArgIsBiggerThanSecondArg() external {
        // It should match the result of `keccak256(abi.encodePacked(b,a))`.
    }
}
<--

success: 1 issue fixed.
```

Running the command without the `--stdout` flag will overwrite the contents of
the solidity file with the fixes applied. Note that not all issues can be
automatically fixed, and bulloak's output will reflect that.

```text
warn: 13 checks failed (run `bulloak check --fix <.tree files>` to apply 11 fixes)
```

You can skip checking that the modifiers are present by passing the `-m`
(or `--skip--modifiers`) option. This way, `bulloak` will not warn when a
modifier is missing from the generated file.

#### Rules

The following rules are currently implemented:

- A Solidity file matching the spec file must exist and be readable.
  - The spec and the Solidity file match if the difference between their names
    is only `.tree` & `.t.sol`.
- There is a contract in the Solidity file and its name matches the root node of
  the spec.
- Every construct, as it would be generated by `bulloak scaffold`, is present in
  the Solidity file.
- The order of every construct, as it would be generated by `bulloak scaffold`,
  matches the spec order.
  - Any valid Solidity construct is allowed and only constructs that would be
    generated by `bulloak scaffold` are checked. This means that any number of
    extra functions, modifiers, etc. can be added to the file.

### Compiler Errors

Another feature of `bulloak` is reporting errors in your input trees.

For example, say you have a buggy `foo.tree` file, which is missing a `└`
character. Running `bulloak scaffold foo.tree` would report the error like this:

```text
•••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••
bulloak error: unexpected `when` keyword

── when the id references a null stream
   ^^^^

--- (line 2, column 4) ---
file: foo.tree
```

## Trees

`bulloak scaffold` scaffolds Solidity test files based on `.tree` specifications
that follow the
[Branching Tree Technique](https://twitter.com/PaulRBerg/status/1682346315806539776).

Currently, there is on-going
[discussion](https://github.com/alexfertel/bulloak/discussions) on how to handle
different edge-cases to better empower the Solidity community. This section is a
description of the current implementation of the compiler.

### Terminology

- _Condition_: `when/given` branches of a tree.
- _Action_: `it` branches of a tree.
- _Action Description_: Children of an action.

### Spec

Each `tree` file should describe at least one function under test. Trees follow
these rules:

- The first line is the root tree identifier, composed of the contract and
  function names which should be delimited by a double colon.
- `bulloak` expects you to use `├` and `└` characters to denote branches.
- If a branch starts with either `when` or `given`, it is a condition.
  - `when` and `given` are interchangeable.
- If a branch starts with `it`, it is an action.
  - Any child branch an action has is called an action description.
- Keywords are case-insensitive: `it` is the same as `It` and `IT`.
- Anything starting with a `//` is a comment and will be stripped from the
  output.
- Multiple trees can be defined in the same file to describe different functions
  by following the same rules, separating them with two newlines.

Take the following Solidity function:

```solidity
function hashPair(bytes32 a, bytes32 b) private pure returns (bytes32) {
    return a < b ? hash(a, b) : hash(b, a);
}
```

A reasonable spec for the above function would be:

```tree
HashPairTest
├── It should never revert.
├── When first arg is smaller than second arg
│   └── It should match the result of `keccak256(abi.encodePacked(a,b))`.
└── When first arg is bigger than second arg
    └── It should match the result of `keccak256(abi.encodePacked(b,a))`.
```

There is a top-level action that will generate a test to check the function
invariant that it should never revert.

Then, we have the two possible preconditions: `a < b` and `a >= b`. Both
branches end in an action that will make `bulloak scaffold` generate the
respective test.

Note the following things:

- Actions are written with ending dots but conditions are not. This is because
  actions support any character, but conditions don't. Since conditions are
  transformed into modifiers, they have to be valid Solidity identifiers.
- You can have top-level actions without conditions. Currently, `bulloak` also
  supports actions with sibling conditions, but this might get removed in a
  future version per this
  [discussion](https://github.com/alexfertel/bulloak/issues/22).
- The root of the tree will be emitted as the name of the test contract.

Suppose you have additional Solidity functions that you want to test in the same
test contract, say `Utils` within `utils.t.sol`:

```solidity
function min(uint256 a, uint256 b) private pure returns (uint256) {
    return a < b ? a : b;
}

function max(uint256 a, uint256 b) private pure returns (uint256) {
    return a > b ? a : b;
}
```

The full spec for all the above functions would be:

```tree
Utils::hashPair
├── It should never revert.
├── When first arg is smaller than second arg
│   └── It should match the result of `keccak256(abi.encodePacked(a,b))`.
└── When first arg is bigger than second arg
    └── It should match the result of `keccak256(abi.encodePacked(b,a))`.


Utils::min
├── It should never revert.
├── When first arg is smaller than second arg
│   └── It should match the value of `a`.
└── When first arg is bigger than second arg
    └── It should match the value of `b`.


Utils::max
├── It should never revert.
├── When first arg is smaller than second arg
│   └── It should match the value of `b`.
└── When first arg is bigger than second arg
    └── It should match the value of `a`.
```

Note the following things:

- Contract identifiers must be present in all roots.
- Contract identifiers that are missing from subsequent trees, or otherwise
  mismatched from the first tree root identifier, will cause `bulloak` to error.
  This violation is not currently fixable with `bulloak check --fix` so will
  need to be manually corrected.
- Duplicate conditions between separate trees will be deduplicated when
  transformed into Solidity modifiers.
- The function part of the root identifier for each tree will be emitted as part
  of the name of the Solidity test (e.g. `test_MinShouldNeverRevert`).

## Output

There are a few things to keep in mind about the scaffolded Solidity test:

- The contract filename is the same as the `.tree` but with a `.t.sol`
  extension. E.g. `test.tree` would correspond to `test.t.sol`.
- Tests are emitted in the order their corresponding actions appear in the
  `.tree` file.
- We generate one modifier per condition, except for leaf condition nodes.
- Test names follow
  [Foundry's best practices](https://book.getfoundry.sh/tutorials/best-practices?highlight=best#tests).

## License

This project is licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  https://www.apache.org/licenses/LICENSE-2.0).
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  https://opensource.org/licenses/MIT).
