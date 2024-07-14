use unicode_xid::UnicodeXID;

use crate::constants::TREES_SEPARATOR;

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
        .replace(|c: char| !c.is_xid_continue() && c != ' ', "")
}

/// Converts a sentence to pascal case.
///
/// The conversion is done by capitalizing the first letter of each word
/// in the title and removing the spaces. For example, the sentence
/// `when only owner` is converted to the `WhenOnlyOwner` string.
pub(crate) fn to_pascal_case(sentence: &str) -> String {
    sentence.split_whitespace().map(capitalize_first_letter).collect::<String>()
}

pub(crate) fn repeat_str(s: &str, n: usize) -> String {
    s.repeat(n)
}

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

/// Splits the input text into distinct trees, delimited by two consecutive
/// newlines.
#[inline]
pub(crate) fn split_trees(text: &str) -> Box<dyn Iterator<Item = &str> + '_> {
    if text.trim().is_empty() {
        return Box::new(std::iter::once(""));
    }

    let trees = text.split(TREES_SEPARATOR).map(str::trim);
    let non_empty_trees = trees.filter(|s| !s.is_empty());
    let no_isolated_comments =
        non_empty_trees.filter(|tree| !only_comments(tree));

    Box::new(no_isolated_comments)
}

fn only_comments(tree: &str) -> bool {
    tree.lines().all(|l| l.trim().starts_with("//"))
}

#[cfg(test)]
mod tests {
    use super::{split_trees, to_pascal_case};

    #[test]
    fn to_modifier() {
        assert_eq!(to_pascal_case("when only owner"), "WhenOnlyOwner");
        assert_eq!(to_pascal_case("when"), "When");
        assert_eq!(to_pascal_case(""), "");
    }

    #[test]
    fn splits_trees() {
        let test_cases = vec![
            ("Foo_Test\n└── when something bad happens\n   └── it should revert", vec![
                "Foo_Test\n└── when something bad happens\n   └── it should revert",
            ]),
            ("Foo_Test\n└── when something bad happens\n   └── it should revert\n\nFoo_Test2\n└── when something bad happens\n   └── it should revert", vec![
                "Foo_Test\n└── when something bad happens\n   └── it should revert",
                "Foo_Test2\n└── when something bad happens\n   └── it should revert",
            ]),
            // Test with varying numbers of newlines between tree splits
            // Assumes behavior is the same for 2 or more newlines
            ("Foo_Test\n└── when something bad happens\n   └── it should revert\n\n\nFoo_Test2\n└── when something bad happens\n   └── it should revert", vec![
                "Foo_Test\n└── when something bad happens\n   └── it should revert",
                "Foo_Test2\n└── when something bad happens\n   └── it should revert",
            ]),
            ("Foo_Test\n└── when something bad happens\n   └── it should revert\n\n\n\nFoo_Test2\n└── when something bad happens\n   └── it should revert", vec![
                "Foo_Test\n└── when something bad happens\n   └── it should revert",
                "Foo_Test2\n└── when something bad happens\n   └── it should revert",
            ]),
            ("Foo_Test\n└── when something bad happens\n   └── it should revert\n\n\n\n\nFoo_Test2\n└── when something bad happens\n   └── it should revert", vec![
                "Foo_Test\n└── when something bad happens\n   └── it should revert",
                "Foo_Test2\n└── when something bad happens\n   └── it should revert",
            ]),
        ];

        for (input, expected) in test_cases {
            let trees = split_trees(input);
            let results: Vec<_> = trees.collect();
            assert_eq!(results, expected, "Failed on input: {}", input);
        }
    }
}
