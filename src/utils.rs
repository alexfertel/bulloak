pub fn capitalize_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub fn lower_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
    }
}

/// This functions makes the appropriate changes to a string to
/// make it a valid identifier.
pub fn sanitize(identifier: &str) -> String {
    identifier.replace('-', "_").replace(['\'', '"'], "")
}

/// Converts a sentence to pascal case.
///
/// The conversion is done by capitalizing the first letter of each word
/// in the title and removing the spaces. For example, the sentence
/// `when only owner` is converted to the `WhenOnlyOwner` string.
pub fn to_pascal_case(sentence: &str) -> String {
    sentence
        .split_whitespace()
        .map(capitalize_first_letter)
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::to_pascal_case;

    #[test]
    fn test_to_modifier() {
        assert_eq!(to_pascal_case("when only owner"), "WhenOnlyOwner");
        assert_eq!(to_pascal_case("when"), "When");
        assert_eq!(to_pascal_case(""), "");
    }
}
