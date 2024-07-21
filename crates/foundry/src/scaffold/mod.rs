//! Defines the `bulloak scaffold` command.
//!
//! This command scaffolds a Solidity file from a spec `.tree` file.

use forge_fmt::fmt;

use crate::{config::Config, hir::translate, sol};

pub mod emitter;
pub mod modifiers;

/// Generates Solidity code from a `.tree` file.
///
/// This function takes the content of a `.tree` file and a configuration,
/// translates it to an intermediate representation, then to Solidity, and
/// finally formats the resulting Solidity code.
pub fn scaffold(text: &str, cfg: &Config) -> anyhow::Result<String> {
    let hir = translate(text, cfg)?;
    let pt = sol::Translator::new(cfg).translate(&hir);
    let source = sol::Formatter::new().emit(pt);
    let formatted =
        fmt(&source).expect("should format the emitted solidity code");

    Ok(formatted)
}
