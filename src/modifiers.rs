use std::collections::HashMap;

use crate::{
    ast::{self, Ast},
    utils::capitalize_first_letter,
    visitor::Visitor,
};

/// Depth-first AST visitor that discovers modifiers.
///
/// Modifiers are discovered by visiting the AST and collecting all condition titles.
/// The collected titles are then converted to modifiers. For example, the title
/// `only owner` is converted to the `whenOnlyOwner` modifier.
///
/// For ease of retrieval, the discovered modifiers are stored in a `HashMap`
/// for the later phases of the compiler. Note that this means that
/// the order of the modifiers is not preserved and that we assume that
/// duplicate titles translate to the same modifier.
pub struct ModifierDiscoverer {
    modifiers: HashMap<String, String>,
}

impl ModifierDiscoverer {
    pub fn new() -> Self {
        Self {
            modifiers: HashMap::new(),
        }
    }

    pub fn discover(&mut self, ast: &ast::Ast) -> &HashMap<String, String> {
        match ast {
            Ast::Root(root) => {
                self.visit_root(root).unwrap();
                &self.modifiers
            }
            _ => unreachable!(),
        }
    }
}

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
        self.modifiers
            .insert(condition.title.clone(), to_modifier(&condition.title));

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

fn to_modifier(title: &str) -> String {
    title
        .split_whitespace()
        .enumerate()
        .map(|(idx, s)| {
            if idx > 0 {
                capitalize_first_letter(s)
            } else {
                s.to_string()
            }
        })
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;

    use crate::error::Result;
    use crate::modifiers::{to_modifier, ModifierDiscoverer};
    use crate::parser::Parser;
    use crate::tokenizer::Tokenizer;

    #[test]
    fn test_to_modifier() {
        assert_eq!(to_modifier("when only owner"), "whenOnlyOwner");
        assert_eq!(to_modifier("when"), "when");
        assert_eq!(to_modifier(""), "");
    }

    #[test]
    fn test_one_child() -> Result<()> {
        let file_contents =
            String::from("file.sol\n└── when something bad happens\n   └── it should revert");

        let tokens = Tokenizer::new().tokenize(&file_contents)?;
        let ast = Parser::new().parse(&file_contents, &tokens)?;
        let mut discoverer = ModifierDiscoverer::new();
        let modifiers = discoverer.discover(&ast);

        assert_eq!(
            modifiers,
            &HashMap::from([(
                "when something bad happens".to_string(),
                "whenSomethingBadHappens".to_string()
            )])
        );

        Ok(())
    }

    #[test]
    fn test_two_children() -> Result<()> {
        let file_contents = String::from(
            r"two_children.t.sol
├── when stuff called
│  └── it should revert
└── when not stuff called
   └── it should revert",
        );

        let tokens = Tokenizer::new().tokenize(&file_contents)?;
        let ast = Parser::new().parse(&file_contents, &tokens)?;
        let mut discoverer = ModifierDiscoverer::new();
        let modifiers = discoverer.discover(&ast);

        assert_eq!(
            modifiers,
            &HashMap::from([
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

        Ok(())
    }

    #[test]
    fn test_deep_tree() -> Result<()> {
        let file_contents = String::from(
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
        );

        let tokens = Tokenizer::new().tokenize(&file_contents)?;
        let ast = Parser::new().parse(&file_contents, &tokens)?;
        let mut discoverer = ModifierDiscoverer::new();
        let modifiers = discoverer.discover(&ast);

        assert_eq!(
            modifiers,
            &HashMap::from([
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

        Ok(())
    }
}
