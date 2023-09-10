// FIXME: Fix doctests in the README.
// #![doc = include_str!("../README.md")]
#![warn(missing_docs, unreachable_pub, unused, rust_2021_compatibility)]
#![warn(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]
pub mod check;
#[doc(hidden)]
pub mod cli;
pub mod parse;
pub mod scaffold;
#[doc(hidden)]
mod utils;
