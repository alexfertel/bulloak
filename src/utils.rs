pub(crate) fn capitalize_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub(crate) fn lower_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
    }
}

/// This function makes the appropriate changes to a string to
/// make it a valid identifier.
pub(crate) fn sanitize(identifier: &str) -> String {
    identifier
        .replace('-', "_")
        .replace(['\'', '"', '.', '{', '}', '’'], "")
}

/// Converts a sentence to pascal case.
///
/// The conversion is done by capitalizing the first letter of each word
/// in the title and removing the spaces. For example, the sentence
/// `when only owner` is converted to the `WhenOnlyOwner` string.
pub(crate) fn to_pascal_case(sentence: &str) -> String {
    sentence
        .split_whitespace()
        .map(capitalize_first_letter)
        .collect::<String>()
}

pub(crate) fn repeat_str(s: &str, n: usize) -> String {
    s.repeat(n)
}

pub(crate) fn pluralize<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
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
