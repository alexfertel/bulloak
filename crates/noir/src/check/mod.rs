//! Validation rules for Noir tests.

pub mod rules;
pub mod violation;

use std::path::Path;

use anyhow::Result;
pub use violation::Violation;

use crate::Config;

/// Check that a Noir test file matches its tree specification.
///
/// # Errors
///
/// Returns an error if checking fails.
pub fn check(tree_path: &Path, cfg: &Config) -> Result<Vec<Violation>> {
    rules::structural_match::check(tree_path, cfg)
}
