#![doc = include_str!("../../../README.md")]
#![warn(missing_docs, unreachable_pub, unused, rust_2021_compatibility)]
#![warn(clippy::all, clippy::pedantic, clippy::cargo)]
pub mod check;
pub mod config;
pub mod hir;
pub mod scaffold;
pub(crate) mod sol;
pub mod syntax;

mod constants;
#[doc(hidden)]
mod utils;

pub(crate) mod error;
pub(crate) mod span;
