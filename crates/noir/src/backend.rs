//! Implementation of the Backend trait for Noir.

use anyhow::Result;
use std::path::PathBuf;

use bulloak_backend::Backend;

use crate::{scaffold, Config};

/// Noir backend implementation.
pub struct NoirBackend {
    config: Config,
}

impl NoirBackend {
    /// Creates a new NoirBackend instance from CLI configuration.
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

impl Backend for NoirBackend {
    fn scaffold(&self, text: &str) -> Result<String> {
        // TODO: make this take an &str for consistency
        scaffold(&text.to_string(), &self.config)
    }

    fn test_filename(&self, tree_file: &PathBuf) -> Result<PathBuf> {
        let stem = tree_file.file_stem().and_then(|s| s.to_str()).ok_or(
            anyhow::anyhow!("invalid filename: {}", tree_file.display()),
        )?;
        let output_filename = format!("{}_test", stem);
        Ok(tree_file.with_file_name(output_filename).with_extension("nr"))
    }
}
