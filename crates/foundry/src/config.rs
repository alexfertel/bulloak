//! `bulloak-core`'s configuration.

use std::path::{Path, PathBuf};

use solang_forge_fmt::{format_to, parse, FormatterConfig, FormatterError};

use crate::constants::DEFAULT_SOL_VERSION;

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
    /// Whether to add `vm.skip(true)` at the beginning of each test.
    pub emit_vm_skip: bool,
    /// Whether to capitalize and punctuate branch descriptions.
    pub format_descriptions: bool,
    /// Formatter configuration resolved from `foundry.toml`.
    pub fmt_config: FormatterConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            files: vec![],
            solidity_version: DEFAULT_SOL_VERSION.to_owned(),
            emit_vm_skip: false,
            skip_modifiers: false,
            format_descriptions: false,
            fmt_config: FormatterConfig::default(),
        }
    }
}

/// Parses and formats Solidity source code using the given formatter config.
///
/// # Errors
///
/// Returns a [`FormatterError`] if the Solidity source cannot be parsed or
/// formatted by `solang_forge_fmt`.
pub fn format_source(
    src: &str,
    config: FormatterConfig,
) -> Result<String, FormatterError> {
    let parsed =
        parse(src).map_err(|_| FormatterError::Fmt(std::fmt::Error))?;
    let mut output = String::new();
    format_to(&mut output, parsed, config)?;
    Ok(output)
}

/// Resolves the `[fmt]` section of the nearest `foundry.toml` by walking up
/// from `start_dir`. Returns `FormatterConfig::default()` if no config is
/// found or if parsing fails.
pub fn resolve_fmt_config(start_dir: &Path) -> FormatterConfig {
    let mut dir = if start_dir.is_file() {
        start_dir.parent().map_or_else(|| PathBuf::from("."), Path::to_path_buf)
    } else {
        start_dir.to_path_buf()
    };

    loop {
        let candidate = dir.join("foundry.toml");
        if candidate.is_file() {
            return match parse_foundry_toml(&candidate) {
                Ok(cfg) => cfg,
                Err(e) => {
                    eprintln!("warning: ignoring {}: {e}", candidate.display());
                    FormatterConfig::default()
                }
            };
        }
        if !dir.pop() {
            break;
        }
    }

    FormatterConfig::default()
}

/// Parses the `[fmt]` section from a `foundry.toml` file, merging user
/// values on top of defaults so that partial configs work correctly.
fn parse_foundry_toml(path: &Path) -> Result<FormatterConfig, String> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    let doc: toml::Table = toml::from_str(&contents)
        .map_err(|e| format!("failed to parse {}: {e}", path.display()))?;

    let Some(user_fmt) = doc.get("fmt") else {
        return Ok(FormatterConfig::default());
    };

    let user_table = user_fmt
        .as_table()
        .ok_or_else(|| "expected [fmt] to be a table".to_string())?;

    // Serialize defaults to a TOML table, then overlay user values.
    let mut merged: toml::Table = {
        let s = toml::to_string(&FormatterConfig::default())
            .map_err(|e| format!("internal error: {e}"))?;
        toml::from_str(&s).map_err(|e| format!("internal error: {e}"))?
    };

    for (k, v) in user_table {
        merged.insert(k.clone(), v.clone());
    }

    toml::Value::Table(merged).try_into().map_err(|e: toml::de::Error| {
        format!("failed to apply fmt config: {e}")
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn resolve_reads_bracket_spacing_from_foundry_toml() {
        let td = tempdir().unwrap();
        fs::write(
            td.path().join("foundry.toml"),
            "[fmt]\nbracket_spacing = true\n",
        )
        .unwrap();

        let cfg = resolve_fmt_config(td.path());
        assert!(cfg.bracket_spacing);
    }

    #[test]
    fn resolve_defaults_when_no_foundry_toml() {
        let td = tempdir().unwrap();
        let cfg = resolve_fmt_config(td.path());
        assert_eq!(cfg, FormatterConfig::default());
    }

    #[test]
    fn resolve_defaults_when_no_fmt_section() {
        let td = tempdir().unwrap();
        fs::write(
            td.path().join("foundry.toml"),
            "[profile.default]\nsrc = \"src\"\n",
        )
        .unwrap();

        let cfg = resolve_fmt_config(td.path());
        assert_eq!(cfg, FormatterConfig::default());
    }

    #[test]
    fn resolve_walks_up_directories() {
        let td = tempdir().unwrap();
        let sub = td.path().join("test").join("nested");
        fs::create_dir_all(&sub).unwrap();
        fs::write(td.path().join("foundry.toml"), "[fmt]\nline_length = 80\n")
            .unwrap();

        let cfg = resolve_fmt_config(&sub);
        assert_eq!(cfg.line_length, 80);
    }

    #[test]
    fn resolve_partial_fmt_section_uses_defaults_for_rest() {
        let td = tempdir().unwrap();
        fs::write(
            td.path().join("foundry.toml"),
            "[fmt]\nbracket_spacing = true\n",
        )
        .unwrap();

        let cfg = resolve_fmt_config(td.path());
        assert!(cfg.bracket_spacing);
        // Other fields should be defaults.
        assert_eq!(cfg.line_length, FormatterConfig::default().line_length);
        assert_eq!(cfg.tab_width, FormatterConfig::default().tab_width);
    }

    #[test]
    fn resolve_prefers_nearest_foundry_toml_for_file_paths() {
        let td = tempdir().unwrap();
        let nested = td.path().join("src").join("nested");
        fs::create_dir_all(&nested).unwrap();
        fs::write(td.path().join("foundry.toml"), "[fmt]\nline_length = 80\n")
            .unwrap();
        fs::write(nested.join("foundry.toml"), "[fmt]\nline_length = 120\n")
            .unwrap();

        let tree_path = nested.join("Spec.tree");
        fs::write(&tree_path, "Foo\n└── It works.\n").unwrap();

        let cfg = resolve_fmt_config(&tree_path);
        assert_eq!(cfg.line_length, 120);
    }
}
