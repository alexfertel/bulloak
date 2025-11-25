//! Implementation of the Backend trait for Foundry/Solidity.

use std::path::PathBuf;

use anyhow::Result;
use bulloak_backend::Backend;

use crate::{config::Config, scaffold::scaffold};

/// Foundry/Solidity backend implementation.
pub struct FoundryBackend {
    config: Config,
}

impl FoundryBackend {
    /// Creates a new FoundryBackend instance from CLI configuration.
    ///
    /// Uses the `From<&Cli>` implementation for `Config` to convert
    /// CLI arguments into the backend's configuration.
    pub fn new<T>(cli: &T) -> Self
    where
        Config: for<'a> From<&'a T>,
    {
        Self { config: cli.into() }
    }
}

impl Backend for FoundryBackend {
    fn scaffold(&self, text: &str) -> Result<String> {
        scaffold(text, &self.config)
    }

    fn test_filename(&self, tree_file: &PathBuf) -> Result<PathBuf> {
        Ok(tree_file.with_extension("t.sol"))
    }
}
