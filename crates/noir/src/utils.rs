//! Utility functions for Noir code generation.

use bulloak_syntax::Ast;

const ROOT_SEPARATOR: &str = "::";

/// Convert a title to snake_case
/// "When user is logged in" ->  "when_user_is_logged_in"
/// "It should return true" -> "it_should_return_true"
pub(crate) fn to_snake_case(title: &str) -> String {
    title
        .trim()
        .chars()
        .filter_map(|c| {
            if c.is_alphanumeric() {
                Some(c.to_ascii_lowercase())
            } else if c.is_whitespace() {
                Some('_')
            } else if c == '_' {
                Some('_')
            } else {
                None
            }
        })
        .collect::<String>()
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

/// Convert {,sub}modules to snake case, but keeping uppercase characters, which
/// are allowable in module names
fn sanitize_module_name(title: &str) -> String {
    title
        .trim()
        .chars()
        .filter_map(|c| {
            if c.is_alphanumeric() {
                Some(c)
            } else if c.is_whitespace() {
                Some('_')
            } else if c == '_' {
                Some('_')
            } else {
                None
            }
        })
        .collect::<String>()
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

/// Extracts the module and submodule name from a root name
pub(crate) fn parse_root_name(
    contract_name: &str,
) -> Result<(String, Option<String>), String> {
    let separators = contract_name.matches(ROOT_SEPARATOR).count();
    if separators > 1 {
        return Err(format!(
            "invalid root \"{}\": expected at most one '{}' separator",
            contract_name, ROOT_SEPARATOR
        ));
    }

    Ok((
        sanitize_module_name(
            contract_name.split(ROOT_SEPARATOR).next().unwrap_or(contract_name),
        ),
        contract_name.split(ROOT_SEPARATOR).nth(1).map(sanitize_module_name),
    ))
}
pub(crate) enum ModuleName {
    Empty,
    Consistent(String),
    Mismatch(String, String),
}
/// Checks that all roots in a multi-root tree have consistent module names.
/// Returns a violation if module names are inconsistent.
/// TODO: move to syntax crate?
pub(crate) fn get_module_name(forest: &[Ast]) -> Result<ModuleName, String> {
    let mut expected_module = ModuleName::Empty;

    for ast in forest {
        let Ast::Root(root) = ast else {
            panic!("expected tree to start with roots, found {:?}", ast);
        };
        let (module_name, _) = parse_root_name(&root.contract_name)?;

        match expected_module {
            ModuleName::Empty => {
                expected_module = ModuleName::Consistent(module_name);
            }
            ModuleName::Consistent(expected) if module_name != expected => {
                return Ok(ModuleName::Mismatch(expected, module_name));
            }
            _ => {}
        }
    }

    Ok(expected_module)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(
            to_snake_case("When user is logged in"),
            "when_user_is_logged_in"
        );
        assert_eq!(
            to_snake_case("It should return true"),
            "it_should_return_true"
        );
        assert_eq!(
            to_snake_case("given amount is zero"),
            "given_amount_is_zero"
        );
        assert_eq!(
            to_snake_case("When first arg is bigger than second arg"),
            "when_first_arg_is_bigger_than_second_arg"
        );
    }

    #[test]
    fn test_to_snake_case_with_special_chars() {
        assert_eq!(to_snake_case("It's working!"), "its_working");
        assert_eq!(to_snake_case("value > 100"), "value_100");
    }

    #[test]
    fn test_to_snake_case_with_underscores() {
        assert_eq!(to_snake_case("It's_working!"), "its_working");
        assert_eq!(to_snake_case("value_is_100"), "value_is_100");
    }

    #[test]
    fn test_sanitize_module_name_with_special_chars() {
        assert_eq!(sanitize_module_name("It's working!"), "Its_working");
        assert_eq!(sanitize_module_name("value > 100"), "value_100");
    }

    #[test]
    fn test_sanitize_module_name_with_underscores() {
        assert_eq!(sanitize_module_name("It's_working!"), "Its_working");
        assert_eq!(sanitize_module_name("value_is_100"), "value_is_100");
    }

    #[test]
    fn test_parse_root_name_with_single_separator() {
        let (module, submodule) =
            parse_root_name("Contract::Submodule").unwrap();
        assert_eq!(module, "Contract");
        assert_eq!(submodule, Some("Submodule".to_string()));
    }

    #[test]
    fn test_parse_root_name_with_no_separator() {
        let (module, submodule) = parse_root_name("Contract").unwrap();
        assert_eq!(module, "Contract");
        assert_eq!(submodule, None);
    }

    #[test]
    fn test_parse_root_name_rejects_multiple_separators() {
        let result = parse_root_name("Contract::A::B");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("expected at most one '::' separator"));
    }
}
