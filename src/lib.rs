// FIXME: Fix doctests in the README.
// #![doc = include_str!("../README.md")]
#![warn(missing_docs, unreachable_pub, unused, rust_2021_compatibility)]
#![warn(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]
pub mod check;
#[doc(hidden)]
pub mod cli;
pub mod hir;
pub mod scaffold;
pub mod syntax;

#[doc(hidden)]
mod utils;

pub(crate) mod error;
pub(crate) mod span;
