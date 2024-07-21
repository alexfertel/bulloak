//! Constants.

/// Default identation used internally.
pub(crate) const INTERNAL_DEFAULT_INDENTATION: usize = 2;
/// Default solidity version used internally.
pub const DEFAULT_SOL_VERSION: &str = "0.8.0";
/// The separator used between contract name and function name when parsing
/// `.tree` files with multiple trees.
pub const CONTRACT_IDENTIFIER_SEPARATOR: &str = "::";
/// The separator used between trees when parsing `.tree` files with multiple
/// trees.
pub const TREES_SEPARATOR: &str = "\n\n";
