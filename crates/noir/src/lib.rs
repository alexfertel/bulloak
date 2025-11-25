//! A `bulloak` backend for aztec-noir tests.
//!
//! `bulloak-noir` provides an implementation of turning a `bulloak-syntax`
//! AST into a `_test.nr` file containing scaffolded and ready-to-run aztec
//! tests.

use anyhow::Result;
pub mod config;
use bulloak_syntax::parse;
pub use config::Config;

/// Generates aztec-noir code from a `.tree` file. Has a uniform interface with the default
/// solidity backend.
/// TODO: should we define a Backend trait?
pub fn scaffold(text: &String, _cfg: &Config) -> Result<String> {
    let _forest = parse(text)?;
    // TODO: do something with the ASTs and config
    Ok("".to_string())
}
