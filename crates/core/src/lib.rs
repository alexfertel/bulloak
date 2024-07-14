#![doc = include_str!("../../../README.md")]
#![warn(missing_docs, unreachable_pub, unused, rust_2021_compatibility)]
#![warn(clippy::all, clippy::pedantic, clippy::cargo)]
pub mod config;
pub mod hir;
pub mod scaffold;
pub mod sol;
pub mod syntax;

pub mod check;
pub use check::violation::{self, Violation, ViolationKind};

mod constants;
#[doc(hidden)]
pub mod utils;

pub(crate) mod error;
pub(crate) mod span;
