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

#[cfg(test)]
mod tests {
    use solang_forge_fmt::FormatterConfig;

    use super::scaffold;
    use crate::config::Config;

    const BASIC_TREE: &str = "\
basic
├── It should never revert.
└── When first arg is smaller than second arg
    ├── When first arg is zero
    │   └── It should do something.
    └── It should match the result of `keccak256(abi.encodePacked(a,b))`.
";

    #[test]
    fn scaffold_emits_contract_and_modifier_for_valid_tree() {
        let output = scaffold(BASIC_TREE, &Config::default()).unwrap();

        assert!(output.contains("contract basic"));
        assert!(output.contains("function test_ShouldNeverRevert() external"));
        assert!(
            output.contains("modifier whenFirstArgIsSmallerThanSecondArg()")
        );
    }

    #[test]
    fn scaffold_adds_forge_std_import_and_vm_skip_when_enabled() {
        let cfg = Config { emit_vm_skip: true, ..Config::default() };
        let output = scaffold(BASIC_TREE, &cfg).unwrap();

        assert!(output.contains("import {Test} from \"forge-std/Test.sol\";"));
        assert!(output.contains("contract basic is Test"));
        assert!(output.contains("vm.skip(true);"));
    }

    #[test]
    fn scaffold_omits_modifiers_when_skip_modifiers_is_enabled() {
        let cfg = Config { skip_modifiers: true, ..Config::default() };
        let output = scaffold(BASIC_TREE, &cfg).unwrap();

        assert!(
            !output.contains("modifier whenFirstArgIsSmallerThanSecondArg()")
        );
        assert!(output.contains("external whenFirstArgIsSmallerThanSecondArg"));
    }

    #[test]
    fn scaffold_returns_error_for_invalid_multi_root_identifier() {
        let invalid_tree = "\
Contract::function::extra
└── It should work.
";

        let err = scaffold(invalid_tree, &Config::default()).unwrap_err();

        assert!(err
            .to_string()
            .contains("too many separators at tree root #1"));
    }

    #[test]
    fn scaffold_respects_bracket_spacing_config() {
        let default_cfg = Config { emit_vm_skip: true, ..Config::default() };
        let output_default = scaffold(BASIC_TREE, &default_cfg).unwrap();
        assert!(output_default
            .contains("import {Test} from \"forge-std/Test.sol\";"));

        let spaced_cfg = Config {
            emit_vm_skip: true,
            fmt_config: FormatterConfig {
                bracket_spacing: true,
                ..FormatterConfig::default()
            },
            ..Config::default()
        };
        let output_spaced = scaffold(BASIC_TREE, &spaced_cfg).unwrap();
        assert!(output_spaced
            .contains("import { Test } from \"forge-std/Test.sol\";"));
    }
}
