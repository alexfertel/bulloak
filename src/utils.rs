use crate::constants::{CONTRACT_PART_SEPARATOR, TREES_SEPARATOR};

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

/// This functions makes the appropriate changes to a string to
/// make it a valid identifier.
pub(crate) fn sanitize(identifier: &str) -> String {
    identifier
        .replace('-', "_")
        .replace(['\'', '"', '.', '{', '}'], "")
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

/// Splits the input text into distinct trees,
/// delimited by two successive newlines.
#[inline]
pub(crate) fn split_trees(text: &str) -> impl Iterator<Item = &str> + '_ {
    text.split(TREES_SEPARATOR)
}

/// Gets the contract name from the HIR tree identifier.
pub(crate) fn get_contract_name_from_identifier(identifier: &str) -> String {
    let contract_name = identifier
        .split(CONTRACT_PART_SEPARATOR)
        .next()
        .expect("should not be empty");
    sanitize(contract_name)
}

#[cfg(test)]
mod tests {
    use super::split_trees;
    use super::to_pascal_case;

    #[test]
    fn to_modifier() {
        assert_eq!(to_pascal_case("when only owner"), "WhenOnlyOwner");
        assert_eq!(to_pascal_case("when"), "When");
        assert_eq!(to_pascal_case(""), "");
    }

    #[test]
    fn splits_trees() {
        let trees =
            split_trees("Foo_Test\n└── when something bad happens\n   └── it should revert");
        assert_eq!(
            trees.collect::<Vec<_>>(),
            vec!["Foo_Test\n└── when something bad happens\n   └── it should revert"]
        );
        let trees =
            split_trees("Foo_Test\n└── when something bad happens\n   └── it should revert\n\nFoo_Test2\n└── when something bad happens\n   └── it should revert");
        assert_eq!(
            trees.collect::<Vec<_>>(),
            vec![
                "Foo_Test\n└── when something bad happens\n   └── it should revert",
                "Foo_Test2\n└── when something bad happens\n   └── it should revert"
            ]
        );
    }
}
