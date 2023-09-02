use indexmap::IndexMap;

use super::{
    ast::{self, Ast},
    utils::{lower_first_letter, to_pascal_case},
    visitor::Visitor,
};

/// AST visitor that discovers modifiers.
///
/// Modifiers are discovered by visiting the AST and collecting all condition titles.
/// The collected titles are then converted to modifiers. For example, the title
/// `when only owner` is converted to the `whenOnlyOwner` modifier.
///
/// For ease of retrieval, the discovered modifiers are stored in a `IndexMap`
/// for the later phases of the compiler. Note that this means that
/// we assume that duplicate titles translate to the same modifier.
/// `IndexMap` was chosen since preserving the order of insertion
/// to match the order of the modifiers in the source tree is helpful
/// and the performance trade-off is dismissable.
pub struct ModifierDiscoverer {
    modifiers: IndexMap<String, String>,
}

impl ModifierDiscoverer {
    /// Create a new discoverer.
    pub fn new() -> Self {
        Self {
            modifiers: IndexMap::new(),
        }
    }

    /// Discover modifiers in the given AST.
    pub fn discover(&mut self, ast: &Ast) -> &IndexMap<String, String> {
        match ast {
            Ast::Root(root) => {
                self.visit_root(root).unwrap();
                &self.modifiers
            }
            _ => unreachable!(),
        }
    }
}

/// A visitor that stores key-value pairs of condition titles and
/// their corresponding modifiers.
impl Visitor for ModifierDiscoverer {
    type Output = ();
    type Error = ();

    fn visit_root(&mut self, root: &ast::Root) -> Result<Self::Output, Self::Error> {
        for condition in &root.asts {
            if let Ast::Condition(condition) = condition {
                self.visit_condition(condition)?;
            }
        }

        Ok(())
    }

    fn visit_condition(&mut self, condition: &ast::Condition) -> Result<Self::Output, Self::Error> {
        self.modifiers.insert(
            condition.title.clone(),
            lower_first_letter(&to_pascal_case(&condition.title)),
        );

        for condition in &condition.asts {
            if let Ast::Condition(condition) = condition {
                self.visit_condition(condition)?;
            }
        }

        Ok(())
    }

    fn visit_action(&mut self, _action: &ast::Action) -> Result<Self::Output, Self::Error> {
        // No-op.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use pretty_assertions::assert_eq;

    use crate::scaffold::{
        error::Result, modifiers::ModifierDiscoverer, parser::Parser, tokenizer::Tokenizer,
    };

    fn discover(file_contents: &str) -> Result<IndexMap<String, String>> {
        let tokens = Tokenizer::new().tokenize(file_contents)?;
        let ast = Parser::new().parse(file_contents, &tokens)?;
        let mut discoverer = ModifierDiscoverer::new();
        discoverer.discover(&ast);

        Ok(discoverer.modifiers)
    }

    #[test]
    fn test_one_child() {
        assert_eq!(
            discover("file.sol\n└── when something bad happens\n   └── it should revert").unwrap(),
            IndexMap::from([(
                "when something bad happens".to_string(),
                "whenSomethingBadHappens".to_string()
            )])
        );
    }

    #[test]
    fn test_two_children() {
        assert_eq!(
            discover(
                r"two_children.t.sol
├── when stuff called
│  └── it should revert
└── when not stuff called
   └── it should revert",
            )
            .unwrap(),
            IndexMap::from([
                (
                    "when stuff called".to_string(),
                    "whenStuffCalled".to_string()
                ),
                (
                    "when not stuff called".to_string(),
                    "whenNotStuffCalled".to_string()
                )
            ])
        );
    }

    #[test]
    fn test_deep_tree() {
        assert_eq!(
            discover(
                r#"deep.sol
├── when stuff called
│  └── it should revert
└── when not stuff called
   ├── when the deposit amount is zero
   │  └── it should revert
   └── when the deposit amount is not zero
      ├── when the number count is zero
      │  └── it should revert
      ├── when the asset is not a contract
      │  └── it should revert
      └── when the asset is a contract
          ├── when the asset misses the ERC_20 return value
          │  ├── it should create the child
          │  ├── it should perform the ERC-20 transfers
          │  └── it should emit a {MultipleChildren} event
          └── when the asset does not miss the ERC_20 return value
              ├── it should create the child
              └── it should emit a {MultipleChildren} event"#,
            )
            .unwrap(),
            IndexMap::from([
                (
                    "when stuff called".to_string(),
                    "whenStuffCalled".to_string()
                ),
                (
                    "when not stuff called".to_string(),
                    "whenNotStuffCalled".to_string()
                ),
                (
                    "when the deposit amount is zero".to_string(),
                    "whenTheDepositAmountIsZero".to_string()
                ),
                (
                    "when the deposit amount is not zero".to_string(),
                    "whenTheDepositAmountIsNotZero".to_string()
                ),
                (
                    "when the number count is zero".to_string(),
                    "whenTheNumberCountIsZero".to_string()
                ),
                (
                    "when the asset is not a contract".to_string(),
                    "whenTheAssetIsNotAContract".to_string()
                ),
                (
                    "when the asset is a contract".to_string(),
                    "whenTheAssetIsAContract".to_string()
                ),
                (
                    "when the asset misses the ERC_20 return value".to_string(),
                    "whenTheAssetMissesTheERC_20ReturnValue".to_string()
                ),
                (
                    "when the asset does not miss the ERC_20 return value".to_string(),
                    "whenTheAssetDoesNotMissTheERC_20ReturnValue".to_string()
                ),
            ])
        );
    }
}
