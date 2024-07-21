/// The separator used between trees when parsing `.tree` files with multiple
/// trees.
pub(crate) const TREES_SEPARATOR: &str = "\n\n";

/// Splits the input text into distinct trees, delimited by two consecutive
/// newlines.
pub(crate) fn split_trees(text: &str) -> Box<dyn Iterator<Item = &str> + '_> {
    if text.trim().is_empty() {
        return Box::new(std::iter::once(""));
    }

    let trees = text.split(TREES_SEPARATOR).map(str::trim);
    let non_empty_trees = trees.filter(|s| !s.is_empty());
    let no_isolated_comments = non_empty_trees.filter(not_only_comments);

    Box::new(no_isolated_comments)
}

/// Return whether the given string only contains lines starting with `//`.
fn not_only_comments(tree: &&str) -> bool {
    !tree.lines().all(|l| l.trim().starts_with("//"))
}

#[cfg(test)]
mod tests {
    use super::split_trees;

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
            // Test with varying numbers of newlines between tree splits.
            // Assumes behavior is the same for 2 or more newlines.
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
