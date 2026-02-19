//! A `bulloak` backend for aztec-noir tests.
//!
//! `bulloak-noir` provides an implementation of turning a `bulloak-syntax`
//! AST into a `_test.nr` file containing scaffolded and ready-to-run aztec
//! tests.

pub mod check;
mod config;
mod constants;
pub mod noir;
mod scaffold;
mod test_structure;
mod utils;

pub use check::check;
pub use config::Config;
pub use scaffold::scaffold;
