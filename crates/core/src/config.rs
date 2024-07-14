//! `bulloak-core`'s configuration.

use std::path::PathBuf;

use crate::constants::INTERNAL_DEFAULT_SOL_VERSION;

/// `bulloak-core`'s configuration.
///
/// Note that configuration coming from the command line is aggregated to this
/// struct only if it makes sense. For example, the `--fix` flag, doesn't make
/// sense in the context of `bulloak-core`.
#[derive(Debug, Clone)]
pub struct Config {
    /// The set of tree files to work on.
    pub files: Vec<PathBuf>,
    /// Whether to emit modifiers.
    pub skip_modifiers: bool,
    /// Sets a Solidity version for the test contracts.
    pub solidity_version: String,
    /// Whether to add `vm.skip(true)` at the begining of each test.
    pub emit_vm_skip: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            files: vec![],
            solidity_version: INTERNAL_DEFAULT_SOL_VERSION.to_owned(),
            emit_vm_skip: false,
            skip_modifiers: false,
        }
    }
}
