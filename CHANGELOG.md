# Changelog

All notable changes to this project will be documented in this file.

## [0.6.2]

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


