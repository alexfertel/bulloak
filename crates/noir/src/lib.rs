//! A `bulloak` backend for aztec-noir tests.
//!
//! `bulloak-noir` provides an implementation of turning a `bulloak-syntax`
//! AST into a `_test.nr` file containing scaffolded and ready-to-run aztec
//! tests.

mod config;
mod scaffold;
mod constants;
mod utils;
pub use scaffold::scaffold;
pub use config::Config;
