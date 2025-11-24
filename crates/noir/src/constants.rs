//! Constants for Noir code generation.

/// Keywords that indicate a test should have the `#[test(should_fail)]`
/// attribute.
pub(crate) const PANIC_KEYWORDS: &[&str] =
    &["it should panic", "it should revert"];

/// Prefix for test functions.
pub(crate) const TEST_PREFIX: &str = "test";
