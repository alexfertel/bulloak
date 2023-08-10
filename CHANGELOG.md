# Changelog

All notable changes to this project will be documented in this file.

## [unreleased]

### Bug Fixes

- *(docs)* Fix typo in README

### Features

- *(docs)* Add logo to README- *(No Category)* Start using git-cliff


### Refactor

- *(docs)* Add missing tag to README

## [0.1.1] - 2023-08-06

### Bug Fixes

- *(README)* Fix typo
- *(tests)* Remove ticks from identifiers- *(No Category)* Support the ~/code/rust character in identifiers
- *(No Category)* Support ticks in identifiers


### Miscellaneous Tasks
- *(No Category)* Update pkg version


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
- *(tokenizer)* Properly parse filenames- *(No Category)* Support all characters in strings for now


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
- *(visitor)* Add the visitor trait- *(No Category)* Add tokenizer
- *(No Category)* Properly setup bin entrypoint
- *(No Category)* Add the --write-files option


### Refactor

- *(ast)* Remove unused Empty variant
- *(emitter)* Rename with_comments -> with_it_as_comments
- *(error)* Update error kind naming convention
- *(fmt)* Fix formatting
- *(semantics)* Simplify types a bit
- *(tests)* Revamp tokenizer tests
- *(tokenizer)* Remove the TokenStream abstraction- *(No Category)* Use full qualifier for the tokenizer mod
- *(No Category)* Rename LICENSE_MIT -> LICENSE-MIT
- *(No Category)* Rework tests across the binary
- *(No Category)* Apply clippy rules
- *(No Category)* Flesh out public API


### Build

- *(ci)* Improve ci workflow- *(No Category)* Add basic github workflow


