# Changelog

All notable changes to this project will be documented in this file.

## [0.8.1] - 2025-04-28

### Bug Fixes

- *(docs)* Set favicon.ico properly
- *(docs)* Make forest animation less CPU-heavy
- *(docs)* Use opengraph-image.png instead of api endpoint (#93)
- *(test)* Ignore a couple of tests on windows
- *(test)* Ignore a couple of scaffold tests on windows
- *(tests)* Make tests work on Windows
- Use appropriate README paths for workspace crates
- Ignore check test on windowns


### Features

- *(bulloak)* Treat input files as globs
- *(docs)* Add landing page (#88)
- *(docs)* Add analytics (#89)
- *(docs)* Improve SEO with robots & sitemap
- *(docs)* Add new logo to the hero section
- *(docs)* Use new logo in project README
- *(docs)* Add links to practical examples to README & docs
- *(docs)* Create open-graph image (#92)
- *(docs)* Add Examples section to README (#96)
- *(test)* Add structural match tests
- Stop panicking from solang_parser errors


### Miscellaneous Tasks

- *(deps)* Bump next from 14.2.6 to 14.2.26 in /docs (#99)
- *(docs)* Polish landing page wording & examples (#90)
- Add tea.yaml constitution
- Add opRetro to FUNDING.json
- Update casing of funding.json


### Refactor

- *(check)* Simplify fix logic & add tests
- Improve readability of the structural match rule
- Remove extra logging


### Build

- *(release)* Init cargo dist


### Ci

- Add a test workflow


## [0.8.0] - 2024-07-27

### Bug Fixes

- Only skip modifiers & not regular fns with -m flag (#82)


### Features

- Revamp repo into publishable crates (#79)


### Refactor

- *(tests)* Use indoc to better format inputs
- Consolidate default sol version between crates
- Rename ast_to_hir -> translate_one


## [0.7.4] - 2024-07-10

### Bug Fixes

- Skip any comment in .tree files (#76)


### Refactor

- *(check)* Use the thiserror crate (#73)
- *(scaffold)* Use the thiserror crate (#71)
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

- *(check)* Simplify function insertion procedures
- *(hir)* Simplify imports
- Improve readability of find_contract
- Rework crate to support configuration (#67)


## [0.7.1] - 2024-04-19

### Bug Fixes

- *(docs)* Properly render version warning in README


### Documentation

- Mention solidity inspector for syntax highlighting (#61)


### Features

- Emit vm.skip(true) when passing -S (#63)


### Miscellaneous Tasks

- *(deps)* Bump h2 from 0.3.22 to 0.3.24 (#59)


### Refactor

- Simplify scaffold entrypoint


### Build

- *(ci)* Fix coverage workflow


## [0.7.0] - 2024-02-25

### Bug Fixes

- *(combine)* Properly handle a few edge cases


### Documentation

- *(README)* Update multiple-roots rules


### Features

- *(combine)* Optimize allocations during the combine pass
- Add support for multiple trees per file (#51)


## [0.6.5] - 2024-02-17

### Bug Fixes

- *(scaffold)* Properly sanitize action titles
- *(scaffold)* Error when a duplicated top action is found


## [0.6.4] - 2024-02-10

### Bug Fixes

- *(check)* Properly measure scratch space length


## [0.6.3] - 2024-01-21

### Bug Fixes

- *(tokenizer)* Enter ident mode when appropriate


## [0.6.2] - 2024-01-18

### Bug Fixes

- Bump zerocopy to apply security patch
- Properly handle apostrophes


### Features

- *(docs)* Add supported by section to README


## [0.6.1] - 2023-12-09

### Bug Fixes

- *(scaffold)* Sanitize contract names before emitting


## [0.6.0] - 2023-12-01

### Bug Fixes

- *(scaffold)* Use forge-fmt to format emitted code
  - **BREAKING**: use forge-fmt to format emitted code
- *(scaffold)* Remove "-i" flag
  - **BREAKING**: remove "-i" flag


### Features

- *(bulloak)* Make output more readable & succint
- Add ‘bulloak check —fix’


## [0.5.4] - 2023-11-16

### Bug Fixes

- *(scaffold)* Error when a duplicated condition is found
- *(tests)* Remove duplicated branches from complex.tree
- Stop ignoring Cargo.lock


### Features

- *(docs)* Add a few comments to structural_match.rs
- *(emitter)* Emit dummy SPDX identifier (#47)


## [0.5.3] - 2023-10-20

### Bug Fixes

- Properly handle a few violation edge-cases


### Miscellaneous Tasks

- *(docs)* Update bulloak's small description


### Refactor

- *(check)* Simplify context creation


## [0.5.2] - 2023-10-05

### Features

- *(check)* Improve violations reporting (#45)
- *(docs)* Add action descriptions to the README


## [0.5.1] - 2023-10-03

### Bug Fixes

- *(docs)* Update README example
- *(docs)* Fix typo in the README


### Features

- *(docs)* Mention VSCode extensions (#36)
- Add support for action descriptions (#44)


### Miscellaneous Tasks

- Ignore .DS_Store files


### Refactor

- Rename solidity -> Solidity
- Rename solidity -> Solidity across the whole project


## [0.5.0] - 2023-09-25

### Bug Fixes

- *(docs)* Remove extra sentence in CONTRIBUTING.md
- *(docs)* Use a smaller logo
- *(docs)* Warn that bulloak is still v0.*.*
- *(scaffold)* Handle empty trees properly (#32)
- *(tokenizer)* Check for identifiers after a given
- Avoid duplicating modifiers unnecessarily


### Features

- *(check)* Print success message if no violations are found
- *(ci)* Add code coverage
- *(docs)* Add codecov badge to README
- Add support for specifying the contract name (#27)
- Add the "bulloak check" command (#31)


## [0.4.5] - 2023-09-01

### Bug Fixes

- *(emitter)* Emit closing brace after visiting actions


## [0.4.4] - 2023-08-31

### Bug Fixes

- *(emitter)* Properly sanitize invalid identifier chars


## [0.4.3] - 2023-08-31

### Bug Fixes

- *(error)* Properly format multiple errors at once (#25)


### Features

- Add support for actions without parent conditions (#26)


## [0.4.2] - 2023-08-29

### Features

- Make keywords case-insensitive (#21)


## [0.4.1] - 2023-08-28

### Bug Fixes

- *(docs)* Add missing flags to README
- Properly check if a file exists before generating


## [0.4.0] - 2023-08-26

### Bug Fixes

- *(bench)* Use the new Scaffolder struct


### Features

- *(cli)* Pass solidity version as an arg (#17)


### Refactor

- *(scaffold)* Add a Scaffolder struct replacing scaffold


## [0.3.0] - 2023-08-25

### Features

- Add support for the 'given' keyword (#16)


## [0.2.2] - 2023-08-24

### Bug Fixes

- *(cli)* Stop overwriting output files (#15)


## [0.2.1] - 2023-08-24

### Bug Fixes

- *(emit)* Use foundry's naming practices for tests (#13)
- *(emit)* Co-locate modifiers with test functions (#14)


### Documentation

- *(README)* Update BTT reference tweet (#5)


### Features

- *(bench)* Add benchmarks using the criterion crate (#6)


## [0.2.0] - 2023-08-10

### Bug Fixes

- *(docs)* Fix typo in README


### Features

- *(docs)* Add logo to README
- Start using git-cliff
- Introduce the Compiled struct to allow setting the output file


### Miscellaneous Tasks

- Generate CHANGELOG.md


### Refactor

- *(docs)* Add missing tag to README


## [0.1.1] - 2023-08-06

### Bug Fixes

- *(README)* Fix typo
- *(tests)* Remove ticks from identifiers
- Support the ~/code/rust character in identifiers
- Support ticks in identifiers


### Miscellaneous Tasks

- Update pkg version


### Refactor

- *(docs)* Update README's example indentation
- *(lint)* Appease clippy


## [0.1.0] - 2023-08-05

### Bug Fixes

- *(cargo)* Update keyword length
- *(ci)* Set proper permissions for clippy
- *(emitter)* Maintain modifier order
- *(emitter)* Remove unnecessary extra space after
- *(parser)* Proper parsing cadence
- *(parser)* Properly parse based on indentation
- *(parser)* Parse filenames with an extension
- *(tokenizer)* Restrict possible character in WHEN/IT blocks
- *(tokenizer)* Start rework to parse filenames and identifiers
- *(tokenizer)* Finish rework to parse filenames and identifiers
- *(tokenizer)* Properly parse filenames
- Support all characters in strings for now


### Features

- *(bulloak)* Initial commit
- *(cargo)* Add missing metadata to Cargo.toml
- *(docs)* Add comments all over the place
- *(docs)* Add commentary to the parser
- *(docs)* Add commentary to the emitter
- *(docs)* Add commentary to semantic analysis
- *(docs)* Add README skeleton
- *(docs)* Add a CONTRIBUTING.md file
- *(docs)* Add documentation to lib.rs
- *(docs)* Update README
- *(docs)* Add doc comment to
- *(emitter)* Add modifier discoverer
- *(emitter)* Add solidity test emitter
- *(emitter)* Add test for edge cases
- *(emitter)* Test emitter flags
- *(error)* Implement nice error formatting
- *(error)* Improve error formatting
- *(parser)* Start parser implementation
- *(semantics)* Add a simple semantic analyzer
- *(tests)* Add unit tests for comments
- *(visitor)* Add the visitor trait
- Add tokenizer
- Properly setup bin entrypoint
- Add the --write-files option


### Refactor

- *(ast)* Remove unused Empty variant
- *(emitter)* Rename with_comments -> with_it_as_comments
- *(error)* Update error kind naming convention
- *(fmt)* Fix formatting
- *(semantics)* Simplify types a bit
- *(tests)* Revamp tokenizer tests
- *(tokenizer)* Remove the TokenStream abstraction
- Use full qualifier for the tokenizer mod
- Rename LICENSE_MIT -> LICENSE-MIT
- Rework tests across the binary
- Apply clippy rules
- Flesh out public API


### Build

- *(ci)* Improve ci workflow
- Add basic github workflow
