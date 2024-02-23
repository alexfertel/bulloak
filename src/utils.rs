use crate::constants::{CONTRACT_PART_SEPARATOR, TREES_SEPARATOR};
use unicode_xid::UnicodeXID;

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

pub(crate) fn split_and_retain_delimiter(s: &str, delimiter: &str) -> Vec<String> {
    if delimiter.is_empty() {
        return vec![s.to_string()];
    }

    let mut result = Vec::new();
    let mut last = 0;
    let mut iter = s.match_indices(delimiter);

    while let Some((index, match_str)) = iter.next() {
        // Push the part before the delimiter, including the delimiter itself
        result.push(s[last..index].to_string() + match_str);
        // Update the last index to start after the current match
        last = index + match_str.len();
    }

    // Push the remaining part of the string, if any
    if last < s.len() {
        result.push(s[last..].to_string());
    } else {
        // If the string ends with the delimiter, add an empty string to signify the split
        result.push("".to_string());
    }

    result
}

/// Splits the input text into distinct trees,
/// delimited by two successive newlines.
#[inline]
pub(crate) fn split_trees(text: &str) -> Box<dyn Iterator<Item = &str> + '_> {
    if text.trim().is_empty() {
        Box::new(std::iter::once(""))
    } else {
        Box::new(
            text.split(TREES_SEPARATOR)
                .map(|s| s.trim())
                .filter(|s| !s.is_empty()),
        )
    }
}

/// Gets the contract name from the HIR tree identifier.
pub(crate) fn get_contract_name_from_identifier(identifier: &str) -> Option<String> {
    identifier
        .split(CONTRACT_PART_SEPARATOR)
        .next()
        .map(sanitize)
        .filter(|s| !s.trim().is_empty())
}

/// Gets the function name from the HIR tree identifier.
pub(crate) fn get_function_name_from_identifier(identifier: &str) -> Option<String> {
    identifier
        .split(CONTRACT_PART_SEPARATOR)
        .nth(1)
        .map(sanitize)
        .filter(|s| !s.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::{split_and_retain_delimiter, split_trees, to_pascal_case};

    #[test]
    fn to_modifier() {
        assert_eq!(to_pascal_case("when only owner"), "WhenOnlyOwner");
        assert_eq!(to_pascal_case("when"), "When");
        assert_eq!(to_pascal_case(""), "");
    }

    #[test]
    fn splits_and_retains_delimiter() {
        let test_cases = vec![
            ("test", "test", vec!["test", ""]),
            ("test", "t", vec!["t", "est", ""]),
            ("test", "e", vec!["te", "st"]),
            ("test", "s", vec!["tes", "t"]),
            ("test", "test", vec!["test", ""]),
            ("test", "", vec!["test"]),
            ("", "test", vec![""]),
        ];

        for (input, delimiter, expected) in test_cases {
            let results = split_and_retain_delimiter(input, delimiter);
            assert_eq!(results, expected, "Failed on input: {}", input);
        }
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
