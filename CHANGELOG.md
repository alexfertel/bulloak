# Changelog

All notable changes to this project will be documented in this file.

## [0.8.1] - 2025-04-28

### Bug Fixes

- _(docs)_ Set favicon.ico properly
- _(docs)_ Make forest animation less CPU-heavy
- _(docs)_ Use opengraph-image.png instead of api endpoint (#93)
- _(test)_ Ignore a couple of tests on windows
- _(test)_ Ignore a couple of scaffold tests on windows
- _(tests)_ Make tests work on Windows
- Use appropriate README paths for workspace crates
- Ignore check test on windows

### Features

- _(bulloak)_ Treat input files as globs
- _(docs)_ Add landing page (#88)
- _(docs)_ Add analytics (#89)
- _(docs)_ Improve SEO with robots & sitemap
- _(docs)_ Add new logo to the hero section
- _(docs)_ Use new logo in project README
- _(docs)_ Add links to practical examples to README & docs
- _(docs)_ Create open-graph image (#92)
- _(docs)_ Add Examples section to README (#96)
- _(test)_ Add structural match tests
- Stop panicking from solang_parser errors

### Miscellaneous Tasks

- _(deps)_ Bump next from 14.2.6 to 14.2.26 in /docs (#99)
- _(docs)_ Polish landing page wording & examples (#90)
- Add tea.yaml constitution
- Add opRetro to FUNDING.json
- Update casing of funding.json

### Refactor

- _(check)_ Simplify fix logic & add tests
- Improve readability of the structural match rule
- Remove extra logging

### Build

- _(release)_ Init cargo dist

### Ci

- Add a test workflow

## [0.8.0] - 2024-07-27

### Bug Fixes

- Only skip modifiers & not regular fns with -m flag (#82)

### Features

- Revamp repo into publishable crates (#79)

### Refactor

- _(tests)_ Use indoc to better format inputs
- Consolidate default sol version between crates
- Rename ast_to_hir -> translate_one

## [0.7.4] - 2024-07-10

### Bug Fixes

- Skip any comment in .tree files (#76)

### Refactor

- _(check)_ Use the thiserror crate (#73)
- _(scaffold)_ Use the thiserror crate (#71)
- Revamp the scaffold entrypoint (#74)

## [0.7.3] - 2024-07-02

### Bug Fixes

- Properly include RevertWhen in test names

### Miscellaneous Tasks

- Add Sablier to the supporters section

## [0.7.2] - 2024-06-10

### Bug Fixes

- Add email to Cargo.toml

### Features

- Add --skip-modifiers flag (#68)

### Miscellaneous Tasks

- Update lock file
- Add FUNDING.json

### Refactor

- _(check)_ Simplify function insertion procedures
- _(hir)_ Simplify imports
- Improve readability of find_contract
- Rework crate to support configuration (#67)

## [0.7.1] - 2024-04-19

### Bug Fixes

- _(docs)_ Properly render version warning in README

### Documentation

- Mention solidity inspector for syntax highlighting (#61)

### Features

- Emit vm.skip(true) when passing -S (#63)

### Miscellaneous Tasks

- _(deps)_ Bump h2 from 0.3.22 to 0.3.24 (#59)

### Refactor

- Simplify scaffold entrypoint

### Build

- _(ci)_ Fix coverage workflow

## [0.7.0] - 2024-02-25

### Bug Fixes

- _(combine)_ Properly handle a few edge cases

### Documentation

- _(README)_ Update multiple-roots rules

### Features

- _(combine)_ Optimize allocations during the combine pass
- Add support for multiple trees per file (#51)

## [0.6.5] - 2024-02-17

### Bug Fixes

- _(scaffold)_ Properly sanitize action titles
- _(scaffold)_ Error when a duplicated top action is found

## [0.6.4] - 2024-02-10

### Bug Fixes

- _(check)_ Properly measure scratch space length

## [0.6.3] - 2024-01-21

### Bug Fixes

- _(tokenizer)_ Enter ident mode when appropriate

## [0.6.2] - 2024-01-18

### Bug Fixes

- Bump zerocopy to apply security patch
- Properly handle apostrophes

### Features

- _(docs)_ Add supported by section to README

## [0.6.1] - 2023-12-09

### Bug Fixes

- _(scaffold)_ Sanitize contract names before emitting

## [0.6.0] - 2023-12-01

### Bug Fixes

- _(scaffold)_ Use forge-fmt to format emitted code
  - **BREAKING**: use forge-fmt to format emitted code
- _(scaffold)_ Remove "-i" flag
  - **BREAKING**: remove "-i" flag

### Features

- _(bulloak)_ Make output more readable & succinct
- Add ‘bulloak check —fix’

## [0.5.4] - 2023-11-16

### Bug Fixes

- _(scaffold)_ Error when a duplicated condition is found
- _(tests)_ Remove duplicated branches from complex.tree
- Stop ignoring Cargo.lock

### Features

- _(docs)_ Add a few comments to structural_match.rs
- _(emitter)_ Emit dummy SPDX identifier (#47)

## [0.5.3] - 2023-10-20

### Bug Fixes

- Properly handle a few violation edge-cases

### Miscellaneous Tasks

- _(docs)_ Update bulloak's small description

### Refactor

- _(check)_ Simplify context creation

## [0.5.2] - 2023-10-05

### Features

- _(check)_ Improve violations reporting (#45)
- _(docs)_ Add action descriptions to the README

## [0.5.1] - 2023-10-03

### Bug Fixes

- _(docs)_ Update README example
- _(docs)_ Fix typo in the README

### Features

- _(docs)_ Mention VSCode extensions (#36)
- Add support for action descriptions (#44)

### Miscellaneous Tasks

- Ignore .DS_Store files

### Refactor

- Rename solidity -> Solidity
- Rename solidity -> Solidity across the whole project

## [0.5.0] - 2023-09-25

### Bug Fixes

- _(docs)_ Remove extra sentence in CONTRIBUTING.md
- _(docs)_ Use a smaller logo
- _(docs)_ Warn that bulloak is still v0._._
- _(scaffold)_ Handle empty trees properly (#32)
- _(tokenizer)_ Check for identifiers after a given
- Avoid duplicating modifiers unnecessarily

### Features

- _(check)_ Print success message if no violations are found
- _(ci)_ Add code coverage
- _(docs)_ Add codecov badge to README
- Add support for specifying the contract name (#27)
- Add the "bulloak check" command (#31)

## [0.4.5] - 2023-09-01

### Bug Fixes

- _(emitter)_ Emit closing brace after visiting actions

## [0.4.4] - 2023-08-31

### Bug Fixes

- _(emitter)_ Properly sanitize invalid identifier chars

## [0.4.3] - 2023-08-31

### Bug Fixes

- _(error)_ Properly format multiple errors at once (#25)

### Features

- Add support for actions without parent conditions (#26)

## [0.4.2] - 2023-08-29

### Features

- Make keywords case-insensitive (#21)

## [0.4.1] - 2023-08-28

### Bug Fixes

- _(docs)_ Add missing flags to README
- Properly check if a file exists before generating

## [0.4.0] - 2023-08-26

### Bug Fixes

- _(bench)_ Use the new Scaffolder struct

### Features

- _(cli)_ Pass solidity version as an arg (#17)

### Refactor

- _(scaffold)_ Add a Scaffolder struct replacing scaffold

## [0.3.0] - 2023-08-25

### Features

- Add support for the 'given' keyword (#16)

## [0.2.2] - 2023-08-24

### Bug Fixes

- _(cli)_ Stop overwriting output files (#15)

## [0.2.1] - 2023-08-24

### Bug Fixes

- _(emit)_ Use foundry's naming practices for tests (#13)
- _(emit)_ Co-locate modifiers with test functions (#14)

### Documentation

- _(README)_ Update BTT reference tweet (#5)

### Features

- _(bench)_ Add benchmarks using the criterion crate (#6)

## [0.2.0] - 2023-08-10

### Bug Fixes

- _(docs)_ Fix typo in README

### Features

- _(docs)_ Add logo to README
- Start using git-cliff
- Introduce the Compiled struct to allow setting the output file

### Miscellaneous Tasks

- Generate CHANGELOG.md

### Refactor

- _(docs)_ Add missing tag to README

## [0.1.1] - 2023-08-06

### Bug Fixes

- _(README)_ Fix typo
- _(tests)_ Remove ticks from identifiers
- Support the ~/code/rust character in identifiers
- Support ticks in identifiers

### Miscellaneous Tasks

- Update pkg version

### Refactor

- _(docs)_ Update README's example indentation
- _(lint)_ Appease clippy

## [0.1.0] - 2023-08-05

### Bug Fixes

- _(cargo)_ Update keyword length
- _(ci)_ Set proper permissions for clippy
- _(emitter)_ Maintain modifier order
- _(emitter)_ Remove unnecessary extra space after
- _(parser)_ Proper parsing cadence
- _(parser)_ Properly parse based on indentation
- _(parser)_ Parse filenames with an extension
- _(tokenizer)_ Restrict possible character in WHEN/IT blocks
- _(tokenizer)_ Start rework to parse filenames and identifiers
- _(tokenizer)_ Finish rework to parse filenames and identifiers
- _(tokenizer)_ Properly parse filenames
- Support all characters in strings for now

### Features

- _(bulloak)_ Initial commit
- _(cargo)_ Add missing metadata to Cargo.toml
- _(docs)_ Add comments all over the place
- _(docs)_ Add commentary to the parser
- _(docs)_ Add commentary to the emitter
- _(docs)_ Add commentary to semantic analysis
- _(docs)_ Add README skeleton
- _(docs)_ Add a CONTRIBUTING.md file
- _(docs)_ Add documentation to lib.rs
- _(docs)_ Update README
- _(docs)_ Add doc comment to
- _(emitter)_ Add modifier discoverer
- _(emitter)_ Add solidity test emitter
- _(emitter)_ Add test for edge cases
- _(emitter)_ Test emitter flags
- _(error)_ Implement nice error formatting
- _(error)_ Improve error formatting
- _(parser)_ Start parser implementation
- _(semantics)_ Add a simple semantic analyzer
- _(tests)_ Add unit tests for comments
- _(visitor)_ Add the visitor trait
- Add tokenizer
- Properly setup bin entrypoint
- Add the --write-files option

### Refactor

- _(ast)_ Remove unused Empty variant
- _(emitter)_ Rename with_comments -> with_it_as_comments
- _(error)_ Update error kind naming convention
- _(fmt)_ Fix formatting
- _(semantics)_ Simplify types a bit
- _(tests)_ Revamp tokenizer tests
- _(tokenizer)_ Remove the TokenStream abstraction
- Use full qualifier for the tokenizer mod
- Rename LICENSE_MIT -> LICENSE-MIT
- Rework tests across the binary
- Apply clippy rules
- Flesh out public API

### Build

- _(ci)_ Improve ci workflow
- Add basic github workflow
