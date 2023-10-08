//! Defines rules that Solidity contracts must follow in order to
//! be considered spec compliant.
//!
//! These rules are checked with the `bulloak check` command.

use super::{context::Context, violation::Violation};

pub(crate) mod structural_match;

/// Trait definition for a rule checker object.
///
/// All children modules must export an implementor of this trait.
pub(crate) trait Checker {
    fn check(ctx: &Context) -> Vec<Violation>;
}
