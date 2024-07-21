//! Various-string manipulation utilities.

use unicode_xid::UnicodeXID;

/// Capitalizes the first letter of a given string.
///
/// This function takes a string slice and returns a new `String` with the first letter capitalized.
/// If the string is empty, it returns an empty string.
///
/// # Arguments
///
/// * `s` - A string slice that holds the input string
///
/// # Returns
///
/// A `String` with the first letter capitalized
///
/// # Examples
///
/// ```
/// # use bulloak_syntax::utils::upper_first_letter;
/// let result = upper_first_letter("hello");
/// assert_eq!(result, "Hello");
/// ```
pub fn upper_first_letter(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(char::to_uppercase)
        .map(|first| first.to_string() + c.as_str())
        .unwrap_or_default()
}

/// Converts the first letter of a given string to lowercase.
///
/// This function takes a string slice and returns a new `String` with the first letter in lowercase.
/// If the string is empty, it returns an empty string.
///
/// # Arguments
///
/// * `s` - A string slice that holds the input string
///
/// # Returns
///
/// A `String` with the first letter in lowercase
///
/// # Examples
///
/// ```
/// # use bulloak_syntax::utils::lower_first_letter;
/// let result = lower_first_letter("Hello");
/// assert_eq!(result, "hello");
/// ```
pub fn lower_first_letter(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(char::to_lowercase)
        .map(|first| first.to_string() + c.as_str())
        .unwrap_or_default()
}

/// Sanitizes a string to make it a valid identifier.
///
/// This function replaces hyphens with underscores and removes any characters
/// that are not valid in an identifier according to the Unicode Standard Annex #31.
///
/// # Arguments
///
/// * `identifier` - A string slice that holds the input identifier
///
/// # Returns
///
/// A `String` containing the sanitized identifier
///
/// # Examples
///
/// ```
/// # use bulloak_syntax::utils::sanitize;
/// let result = sanitize("my-variable@123");
/// assert_eq!(result, "my_variable123");
/// ```
pub fn sanitize(identifier: &str) -> String {
    identifier
        .replace('-', "_")
        .replace(|c: char| !c.is_xid_continue() && c != ' ', "")
}

/// Converts a sentence to pascal case.
///
/// The conversion is done by capitalizing the first letter of each word
/// in the title and removing the spaces. For example, the sentence
/// `when only owner` is converted to the `WhenOnlyOwner` string.
///
/// # Arguments
///
/// * `sentence` - A string slice that holds the input sentence
///
/// # Returns
///
/// A `String` in pascal case
///
/// # Examples
///
/// ```
/// # use bulloak_syntax::utils::to_pascal_case;
/// let result = to_pascal_case("when only owner");
/// assert_eq!(result, "WhenOnlyOwner");
/// ```
pub fn to_pascal_case(sentence: &str) -> String {
    sentence.split_whitespace().map(upper_first_letter).collect::<String>()
}

/// Repeats a given string a specified number of times.
///
/// # Arguments
///
/// * `s` - A string slice to be repeated
/// * `n` - The number of times to repeat the string
///
/// # Returns
///
/// A `String` containing the repeated string
///
/// # Examples
///
/// ```
/// # use bulloak_syntax::utils::repeat_str;
/// let result = repeat_str("abc", 3);
/// assert_eq!(result, "abcabcabc");
/// ```
pub fn repeat_str(s: &str, n: usize) -> String {
    s.repeat(n)
}

/// Returns the singular or plural form of a word based on the count.
///
/// # Arguments
///
/// * `count` - The count to determine which form to use
/// * `singular` - The singular form of the word
/// * `plural` - The plural form of the word
///
/// # Returns
///
/// A string slice containing either the singular or plural form
///
/// # Examples
///
/// ```
/// # use bulloak_syntax::utils::pluralize;
/// assert_eq!(pluralize(1, "apple", "apples"), "apple");
/// assert_eq!(pluralize(2, "apple", "apples"), "apples");
/// ```
pub fn pluralize<'a>(
    count: usize,
    singular: &'a str,
    plural: &'a str,
) -> &'a str {
    if count == 1 {
        singular
    } else {
        plural
    }
}

#[cfg(test)]
mod tests {
    use super::to_pascal_case;

    #[test]
    fn to_modifier() {
        assert_eq!(to_pascal_case("when only owner"), "WhenOnlyOwner");
        assert_eq!(to_pascal_case("when"), "When");
        assert_eq!(to_pascal_case(""), "");
    }
}
