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
/// delimited by two successive newlines
///
/// This function is called before the tokenization and parsing steps.
#[inline]
pub(crate) fn split_trees(text: &str) -> Vec<&str> {
    text.split(TREES_SEPARATOR).collect::<Vec<&str>>()
}

/// Gets the contract name from the HIR tree identifier.
///
/// This function is called during verification of the combined HIR.
pub(crate) fn get_contract_name_from_identifier(identifier: &String) -> String {
    let identifier_parts: Vec<&str> = identifier.split(CONTRACT_PART_SEPARATOR).collect();
    return identifier_parts[0].to_string();
}

/// Generates the HIR for a single tree.
///
/// This function leverages `crate::syntax::parse` and `crate::hir::translator::Translator::translate`
/// to hide away most of the complexity of `bulloak`'s internal compiler.
pub(crate) fn translate_tree_to_hir(tree: &str) -> crate::error::Result<crate::hir::Hir> {
    let ast = crate::syntax::parse(tree)?;
    let mut discoverer = crate::scaffold::modifiers::ModifierDiscoverer::new();
    let modifiers = discoverer.discover(&ast);
    Ok(crate::hir::translator::Translator::new().translate(&ast, modifiers))
}

/// High-level function that returns a HIR given the contents of a `.tree` file.
///
/// This function leverages `translate_tree_to_hir` to generate the HIR for each tree,
/// and `crate::hir::combiner::Combiner::combine` to combine the HIRs into a single HIR.
pub(crate) fn translate_and_combine_trees(text: &str) -> crate::error::Result<crate::hir::Hir> {
    let trees = split_trees(text);
    let hirs = trees.iter()
        .map(|tree| translate_tree_to_hir(tree))
        .collect::<crate::error::Result<Vec<crate::hir::Hir>>>()?;
    Ok(crate::hir::combiner::Combiner::new().combine(&hirs)?)
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
    fn test_split_trees() {
        assert_eq!(
            split_trees("Foo_Test\n└── when something bad happens\n   └── it should revert"),
            vec!["Foo_Test\n└── when something bad happens\n   └── it should revert"]
        );
        assert_eq!(
            split_trees("Foo_Test\n└── when something bad happens\n   └── it should revert\n\nFoo_Test2\n└── when something bad happens\n   └── it should revert"),
            vec!["Foo_Test\n└── when something bad happens\n   └── it should revert", "Foo_Test2\n└── when something bad happens\n   └── it should revert"]
        );
    }
}
