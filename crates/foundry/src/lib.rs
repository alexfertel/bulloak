//! A `bulloak` backend for Foundry tests.
//!
//! `bulloak-foundry` provides an implementation of turning a `bulloak-syntax`
//! AST into a `.t.sol` file containing scaffolded and ready-to-run Foundry
//! tests.
//!
//! It also includes the implementation of a system to check that tests
//! correspond to a spec in the form of a `.tree`. This implementation allows
//! for defining rules to be checked, which may be automatically fixed.

pub mod check;
pub mod config;
pub mod constants;
pub mod hir;
pub mod scaffold;
pub mod sol;
pub mod utils;

pub use check::violation::{self, Violation, ViolationKind};
