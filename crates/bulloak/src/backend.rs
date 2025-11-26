//! Bulloak backend trait
//!
//! This module defines the core trait that all bulloak backends must implement,
//! along with their concrete implementations
use std::path::PathBuf;

use anyhow::Result;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::cli::Cli;

/// Trait for backends that generate test files from `.tree` specifications.
///
/// Implementors of this trait must transform a tree specification (in string
/// form) into generated test code for a specific testing framework.
pub trait Backend: Send + Sync {
    /// Scaffolds test code from a tree specification.
    /// Must output it already formatted, as it won't be processed further
    fn scaffold(&self, text: &str) -> Result<String>;

    /// Returns the output test file path for a given tree file path.
    fn test_filename(&self, tree_file: &PathBuf) -> Result<PathBuf>;
}

/// Available backend types for CLI argument parsing.
#[derive(Debug, Serialize, Deserialize, Clone, ValueEnum)]
pub enum BackendKind {
    /// original Foundry backend.
    Solidity,
    Noir,
}

/// Solidity/Foundry backend with baked-in config.
pub(crate) struct SolidityBackend {
    config: bulloak_foundry::config::Config,
}

/// Noir/Aztec backend with baked-in config.
pub(crate) struct NoirBackend {
    config: bulloak_noir::Config,
}

impl BackendKind {
    /// Creates a boxed backend instance with config derived from CLI.
    pub fn get(&self, cli: &Cli) -> Box<dyn Backend> {
        match self {
            Self::Solidity => Box::new(SolidityBackend { config: cli.into() }),
            Self::Noir => Box::new(NoirBackend { config: cli.into() }),
        }
    }
}

impl Backend for SolidityBackend {
    fn scaffold(&self, text: &str) -> Result<String> {
        let emitted = bulloak_foundry::scaffold::scaffold(text, &self.config)?;
        Ok(forge_fmt::fmt(&emitted).unwrap_or(emitted))
    }

    fn test_filename(&self, tree_file: &PathBuf) -> Result<PathBuf> {
        Ok(tree_file.with_extension("t.sol"))
    }
}

impl Backend for NoirBackend {
    fn scaffold(&self, text: &str) -> Result<String> {
        bulloak_noir::scaffold(&text, &self.config)
    }

    fn test_filename(&self, tree_file: &PathBuf) -> Result<PathBuf> {
        let stem = tree_file.file_stem().and_then(|s| s.to_str()).ok_or(
            anyhow::anyhow!("invalid filename: {}", tree_file.display()),
        )?;
        let output_filename = format!("{}_test", stem);
        Ok(tree_file.with_file_name(output_filename).with_extension("nr"))
    }
}
