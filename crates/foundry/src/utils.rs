//! Utility functions.

/// Formats a comment by capitalizing the first letter and adding a period.
///
/// # Panics
///
/// This function will panic if the input string is not empty but contains no
/// valid characters. In practice, this should never happen with normal string
/// input.
#[must_use]
pub fn format_comment(comment: &str) -> String {
    if comment.is_empty() {
        return comment.to_string();
    }

    let mut chars = comment.chars();
    let first_char = chars.next().unwrap();
    let capitalized =
        first_char.to_uppercase().chain(chars).collect::<String>();

    // Add a period if the comment doesn't end with punctuation
    if matches!(capitalized.chars().last(), Some('.' | '!' | '?')) {
        capitalized
    } else {
        format!("{capitalized}.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_comment() {
        assert_eq!(format_comment(""), "");
        assert_eq!(format_comment("it should revert"), "It should revert.");
        assert_eq!(format_comment("emit an event"), "Emit an event.");
        assert_eq!(format_comment("capitalize me!"), "Capitalize me!");
        assert_eq!(format_comment("Hello world."), "Hello world.");
    }
}
