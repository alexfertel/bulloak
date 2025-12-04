//! A `bulloak` backend for aztec-noir tests.
//!
//! `bulloak-noir` provides an implementation of turning a `bulloak-syntax`
//! AST into a `_test.nr` file containing scaffolded and ready-to-run aztec
//! tests.

mod config;
mod constants;
pub mod noir;
mod scaffold;
pub mod check;
mod utils;

pub use config::Config;
pub use scaffold::scaffold;
pub use check::check;
