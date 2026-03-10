//! Defines the `bulloak scaffold` command.
//!
//! This command scaffolds a Solidity file from a spec `.tree` file.

use crate::{config::Config, hir::translate, sol};

pub mod comment;
pub mod emitter;
pub mod modifiers;

/// Generates Solidity code from a `.tree` file.
///
/// This function takes the content of a `.tree` file and a configuration,
/// translates it to an intermediate representation, then to Solidity, and
/// finally formats the resulting Solidity code.
///
/// # Errors
///
/// Returns an error if the tree cannot be translated to HIR or if the emitted
/// Solidity cannot be formatted.
pub fn scaffold(text: &str, cfg: &Config) -> anyhow::Result<String> {
    let hir = translate(text, cfg)?;
    let pt = sol::Translator::new(cfg).translate(&hir);
    let source = sol::Formatter::new().emit(pt);
    let formatted =
        crate::config::format_source(&source, cfg.fmt_config.clone())?;

    Ok(formatted)
}
