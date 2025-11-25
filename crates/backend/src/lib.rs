//! Bulloak backend trait
//!
//! This crate defines the core trait that all bulloak backends must implement.

use anyhow::Result;
use std::path::PathBuf;

/// Trait for backends that generate test files from `.tree` specifications.
///
/// Implementors of this trait can transform a tree specification (in string form)
/// into generated test code for a specific testing framework.
pub trait Backend: Send + Sync {
    /// Scaffolds test code from a tree specification.
    /// The generated test code as a string, or an error if generation fails.
    fn scaffold(&self, text: &str) -> Result<String>;

    /// Returns the output test file path for a given tree file path.
    fn test_filename(&self, tree_file: &PathBuf) -> Result<PathBuf>;
}
