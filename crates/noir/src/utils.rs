//! Utility functions for Noir code generation.

/// Convert a title to snake_case, stripping BDD prefixes.
///
/// # Examples
///
/// ```ignore
/// assert_eq!(to_snake_case("When user is logged in"), "user_is_logged_in");
/// assert_eq!(to_snake_case("It should return true"), "should_return_true");
/// ```
pub(crate) fn to_snake_case(title: &str) -> String {
    // Strip BDD prefixes (case-insensitive)
    let title = title.trim();
    let stripped = title
        .strip_prefix("when ")
        .or_else(|| title.strip_prefix("When "))
        .or_else(|| title.strip_prefix("given "))
        .or_else(|| title.strip_prefix("Given "))
        .or_else(|| title.strip_prefix("it "))
        .or_else(|| title.strip_prefix("It "))
        .unwrap_or(title);

    // Convert to snake_case
    stripped
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(
            to_snake_case("When user is logged in"),
            "user_is_logged_in"
        );
        assert_eq!(
            to_snake_case("It should return true"),
            "should_return_true"
        );
        assert_eq!(to_snake_case("given amount is zero"), "amount_is_zero");
        assert_eq!(
            to_snake_case("When first arg is bigger than second arg"),
            "first_arg_is_bigger_than_second_arg"
        );
    }

    #[test]
    fn test_to_snake_case_with_special_chars() {
        assert_eq!(to_snake_case("It's working!"), "its_working");
        assert_eq!(to_snake_case("value > 100"), "value_100");
    }
}
