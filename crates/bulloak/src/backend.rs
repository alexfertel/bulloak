//! Bulloak backend trait
//!
//! This module defines the core trait that all bulloak backends must implement,
//! along with their concrete implementations
use regex::Regex;
use std::path::PathBuf;
use thiserror::Error;

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

#[derive(Error, Debug)]
enum BackendError {
    #[error("invalid filename: {0}")]
    InvalidFilename(PathBuf),

    #[error("missing .tree extension: {0}")]
    MissingTreeExtension(PathBuf),
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

fn validate_extension(input: &PathBuf) -> Result<(), BackendError> {
    let extension = input
        .extension()
        .ok_or(BackendError::InvalidFilename(input.to_owned()))?;
    if extension != "tree" {
        return Err(BackendError::MissingTreeExtension(input.to_owned()));
    }
    Ok(())
}

impl Backend for SolidityBackend {
    fn scaffold(&self, text: &str) -> Result<String> {
        let emitted = bulloak_foundry::scaffold::scaffold(text, &self.config)?;
        Ok(forge_fmt::fmt(&emitted).unwrap_or(emitted))
    }

    fn test_filename(&self, tree_file: &PathBuf) -> Result<PathBuf> {
        validate_extension(tree_file)?;
        Ok(tree_file.with_extension("t.sol"))
    }
}

impl Backend for NoirBackend {
    fn scaffold(&self, text: &str) -> Result<String> {
        bulloak_noir::scaffold(&text, &self.config)
    }

    fn test_filename(&self, tree_file: &PathBuf) -> Result<PathBuf> {
        let regex = Regex::new(r"\.tree$").unwrap();
        validate_extension(tree_file)?;
        let input_filename = tree_file.to_str().ok_or(anyhow::anyhow!(
            "invalid filename: {}",
            tree_file.display()
        ))?;
        let output_filename = regex.replace_all(input_filename, "_test.nr");
        if output_filename == input_filename {
            return Err(anyhow::anyhow!(
                "invalid filename, {}",
                tree_file.display()
            ));
        }
        Ok(PathBuf::from(output_filename.into_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{path::PathBuf, sync::LazyLock};

    static NOIR_BACKEND: LazyLock<NoirBackend> = LazyLock::new(|| {
        NoirBackend { config: bulloak_noir::Config::default() }
    });
    static FOUNDRY_BACKEND: LazyLock<SolidityBackend> = LazyLock::new(|| {
        SolidityBackend { config: bulloak_foundry::config::Config::default() }
    });

    #[test]
    fn test_simple_tree_file() {
        let input = PathBuf::from("MyContract.tree");
        let result = NOIR_BACKEND.test_filename(&input).unwrap();
        assert_eq!(result, PathBuf::from("MyContract_test.nr"));

        let result = FOUNDRY_BACKEND.test_filename(&input).unwrap();
        assert_eq!(result, PathBuf::from("MyContract.t.sol"));
    }

    #[test]
    fn test_with_directory_path() {
        let input = PathBuf::from("src/contracts/MyContract.tree");
        let result = NOIR_BACKEND.test_filename(&input).unwrap();
        assert_eq!(result, PathBuf::from("src/contracts/MyContract_test.nr"));

        let result = FOUNDRY_BACKEND.test_filename(&input).unwrap();
        assert_eq!(result, PathBuf::from("src/contracts/MyContract.t.sol"));
    }

    #[test]
    fn test_with_multiple_dots() {
        let input = PathBuf::from("My.Complex.Contract.tree");
        let result = NOIR_BACKEND.test_filename(&input).unwrap();
        assert_eq!(result, PathBuf::from("My.Complex.Contract_test.nr"));
        let result = FOUNDRY_BACKEND.test_filename(&input).unwrap();
        assert_eq!(result, PathBuf::from("My.Complex.Contract.t.sol"));
    }

    #[test]
    fn test_already_has_test_suffix() {
        let input = PathBuf::from("MyContract_test.tree");
        let result = NOIR_BACKEND.test_filename(&input).unwrap();
        // Should append another _test
        assert_eq!(result, PathBuf::from("MyContract_test_test.nr"));
        let result = FOUNDRY_BACKEND.test_filename(&input).unwrap();
        assert_eq!(result, PathBuf::from("MyContract_test.t.sol"));
    }

    #[test]
    fn test_with_absolute_path() {
        let input = PathBuf::from("/home/user/project/Contract.tree");
        let result = NOIR_BACKEND.test_filename(&input).unwrap();
        assert_eq!(
            result,
            PathBuf::from("/home/user/project/Contract_test.nr")
        );
        let result = FOUNDRY_BACKEND.test_filename(&input).unwrap();
        assert_eq!(result, PathBuf::from("/home/user/project/Contract.t.sol"));
    }

    #[test]
    fn test_preserves_parent_directories() {
        let input = PathBuf::from("tests/specs/nested/MyTest.tree");
        let result = NOIR_BACKEND.test_filename(&input).unwrap();
        assert_eq!(result, PathBuf::from("tests/specs/nested/MyTest_test.nr"));
        let result = FOUNDRY_BACKEND.test_filename(&input).unwrap();
        assert_eq!(result, PathBuf::from("tests/specs/nested/MyTest.t.sol"));
    }

    #[test]
    fn test_no_extension() {
        let input = PathBuf::from("MyContract");
        let result = NOIR_BACKEND.test_filename(&input);
        assert!(result.is_err());
        let result = FOUNDRY_BACKEND.test_filename(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_extension() {
        let input = PathBuf::from("MyContract.txt");
        let result = NOIR_BACKEND.test_filename(&input);
        assert!(result.is_err());
        let result = FOUNDRY_BACKEND.test_filename(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_fails() {
        let input = PathBuf::from("");
        let result = NOIR_BACKEND.test_filename(&input);
        assert!(result.is_err());
        let result = FOUNDRY_BACKEND.test_filename(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_directory_only_fails() {
        let input = PathBuf::from("src/");
        let result = NOIR_BACKEND.test_filename(&input);
        assert!(result.is_err());
        let result = FOUNDRY_BACKEND.test_filename(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_with_unicode() {
        let input = PathBuf::from("🐻.tree");
        let result = NOIR_BACKEND.test_filename(&input).unwrap();
        assert_eq!(result, PathBuf::from("🐻_test.nr"));
        let result = FOUNDRY_BACKEND.test_filename(&input).unwrap();
        assert_eq!(result, PathBuf::from("🐻.t.sol"));
    }

    #[test]
    fn test_with_spaces() {
        let input = PathBuf::from("My Contract.tree");
        let result = NOIR_BACKEND.test_filename(&input).unwrap();
        assert_eq!(result, PathBuf::from("My Contract_test.nr"));
        let result = FOUNDRY_BACKEND.test_filename(&input).unwrap();
        assert_eq!(result, PathBuf::from("My Contract.t.sol"));
    }

    #[test]
    fn test_only_extension() {
        let input = PathBuf::from(".tree");
        let result = NOIR_BACKEND.test_filename(&input);
        assert!(result.is_err());
        let result = FOUNDRY_BACKEND.test_filename(&input);
        assert!(result.is_err());

        let input = PathBuf::from("/foo/.tree");
        let result = NOIR_BACKEND.test_filename(&input);
        assert!(result.is_err());
        let result = FOUNDRY_BACKEND.test_filename(&input);
        assert!(result.is_err());

        let input = PathBuf::from("src/.tree");
        let result = NOIR_BACKEND.test_filename(&input);
        assert!(result.is_err());
        let result = FOUNDRY_BACKEND.test_filename(&input);
        assert!(result.is_err());
    }
}
