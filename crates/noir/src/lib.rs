//! A `bulloak` backend for aztec-noir tests.
//!
//! `bulloak-noir` provides an implementation of turning a `bulloak-syntax`
//! AST into a `_test.nr` file containing scaffolded and ready-to-run aztec
//! tests.

mod config;
mod constants;
mod scaffold;
mod utils;

pub use config::Config;
pub use scaffold::scaffold;
