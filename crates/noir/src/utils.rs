//! Utility functions for Noir code generation.

use bulloak_syntax::Ast;

/// Convert a title to snake_case, stripping BDD prefixes.
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
pub(crate) fn parse_root_name(contract_name: &str) -> (String, Option<String>) {
    (
        to_snake_case(
            contract_name.split("::").next().unwrap_or(contract_name),
        ),
        contract_name.split("::").nth(1).and_then(|x| Some(to_snake_case(x))),
    )
}
/// Checks that all roots in a multi-root tree have consistent module names.
/// Returns a violation if module names are inconsistent.
/// TODO: move to syntax crate?
pub(crate) fn get_module_name(
    forest: &[Ast],
) -> Option<Result<String, (String, String)>> {
    let mut expected_module: Option<String> = None;

    for ast in forest {
        let Ast::Root(root) = ast else {
            panic!("tree does not start with a root");
        };
        let (module_name, _) = parse_root_name(&root.contract_name);

        match expected_module {
            None => {
                expected_module = Some(module_name);
            }
            Some(expected) if module_name != expected => {
                return Some(Err((expected, module_name)));
            }
            _ => {}
        }
    }

    match expected_module {
        None => None,
        Some(i) => Some(Ok(i)),
    }
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
}
