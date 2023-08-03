use indexmap::IndexMap;
use std::result;

use crate::{
    ast::{self, Ast},
    utils::capitalize_first_letter,
    visitor::Visitor,
};

/// Solidity code emitter.
pub struct Emitter {
    with_comments: bool,
    indent: usize,
}

impl Emitter {
    pub fn new(with_comments: bool, indent: usize) -> Self {
        Self {
            with_comments,
            indent,
        }
    }

    pub fn emit(self, ast: &ast::Ast, modifiers: &IndexMap<String, String>) -> String {
        EmitterI::new(self, modifiers).emit(ast)
    }

    fn indent(&self) -> String {
        " ".repeat(self.indent)
    }
}

struct EmitterI<'a> {
    modifier_stack: Vec<&'a str>,
    modifiers: &'a IndexMap<String, String>,
    emitter: Emitter,
}

impl<'a> EmitterI<'a> {
    fn new(emitter: Emitter, modifiers: &'a IndexMap<String, String>) -> Self {
        Self {
            modifier_stack: Vec::new(),
            modifiers,
            emitter,
        }
    }

    fn emit(&mut self, ast: &ast::Ast) -> String {
        match ast {
            Ast::Root(ref root) => self.visit_root(root).unwrap(),
            _ => unreachable!(),
        }
    }

    fn emit_modifier(&self, modifier: &str) -> String {
        let mut emitted = String::new();
        let indentation = self.emitter.indent();
        emitted.push_str(&format!("{}modifier {}() {{\n", indentation, modifier));
        emitted.push_str(&format!("{}_;\n", indentation.repeat(2)));
        emitted.push_str(&format!("{}}}\n", indentation));
        emitted.push('\n');

        emitted
    }
}

impl<'a> Visitor for EmitterI<'a> {
    type Output = String;
    type Error = ();

    fn visit_root(&mut self, root: &ast::Root) -> result::Result<Self::Output, Self::Error> {
        let mut emitted = String::new();
        emitted.push_str("pragma solidity [VERSION];\n\n");

        // It's fine to unwrap here because we check that the filename always has an extension.
        let contract_name = root.file_name.split_once('.').unwrap().0;
        let contract_name = capitalize_first_letter(contract_name);
        emitted.push_str(format!("contract {}Test {{\n", contract_name).as_str());

        for modifier in self.modifiers.values() {
            emitted.push_str(&self.emit_modifier(modifier));
        }

        for condition in &root.asts {
            if let Ast::Condition(condition) = condition {
                emitted.push_str(&self.visit_condition(condition)?);
            }
        }

        emitted.push('}');

        Ok(emitted)
    }

    fn visit_condition(
        &mut self,
        condition: &ast::Condition,
    ) -> result::Result<Self::Output, Self::Error> {
        let mut emitted = String::new();

        // It's fine to unwrap here because we discover all modifiers in a previous pass.
        let modifier = self.modifiers.get(&condition.title).unwrap();
        self.modifier_stack.push(modifier);

        // We count instead of collecting into a Vec to avoid allocating a Vec for each condition.
        let action_count = condition.asts.iter().filter(|ast| ast.is_action()).count();
        let mut actions = condition.asts.iter().filter(|ast| ast.is_action());

        if action_count > 0 {
            let fn_indentation = self.emitter.indent();
            let fn_body_indentation = fn_indentation.repeat(2);
            // It's fine to unwrap here because we check that no action appears outside of a condition.
            let last_modifier = *self.modifier_stack.last().unwrap();
            let test_name = capitalize_first_letter(last_modifier);

            // If the only action is `it should revert`, we slightly change the function name
            // to reflect this.
            let is_revert = action_count == 1
                && actions.next().is_some_and(|a| {
                    if let Ast::Action(action) = a {
                        action.title == "it should revert"
                    } else {
                        false
                    }
                });
            let function_name = if is_revert {
                format!("testReverts{}", test_name)
            } else {
                format!("test{}", test_name)
            };
            emitted.push_str(format!("{}function {}()\n", fn_indentation, function_name).as_str());
            emitted.push_str(format!("{}external \n", fn_body_indentation).as_str());

            for modifier in &self.modifier_stack {
                emitted.push_str(format!("{}{}\n", fn_body_indentation, modifier).as_str());
            }

            emitted.push_str(format!("{}{{\n", fn_indentation).as_str());
        }

        for action in &condition.asts {
            if let Ast::Action(action) = action {
                emitted.push_str(&self.visit_action(action)?);
            }
        }

        for condition in &condition.asts {
            if let Ast::Condition(condition) = condition {
                emitted.push_str(&self.visit_condition(condition)?);
            }
        }

        if action_count > 0 {
            emitted.push_str(format!("{}}}\n\n", self.emitter.indent()).as_str());
        }

        self.modifier_stack.pop();

        Ok(emitted)
    }

    fn visit_action(&mut self, action: &ast::Action) -> result::Result<Self::Output, Self::Error> {
        let mut emitted = String::new();

        if self.emitter.with_comments {
            let indentation = self.emitter.indent().repeat(2);
            emitted.push_str(format!("{}// {}\n", indentation, action.title).as_str());
        }

        Ok(emitted)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::emitter;
    use crate::error::Result;
    use crate::modifiers;
    use crate::parser::Parser;
    use crate::tokenizer::Tokenizer;

    fn scaffold(text: &str) -> Result<String> {
        let tokens = Tokenizer::new().tokenize(&text)?;
        let ast = Parser::new().parse(&text, &tokens)?;
        let mut discoverer = modifiers::ModifierDiscoverer::new();
        let modifiers = discoverer.discover(&ast);
        Ok(emitter::Emitter::new(true, 2).emit(&ast, &modifiers))
    }

    #[test]
    fn test_one_child() -> Result<()> {
        let file_contents =
            String::from("file.sol\n└── when something bad happens\n   └── it should not revert");

        assert_eq!(
            &scaffold(&file_contents)?,
            r"pragma solidity [VERSION];

contract FileTest {
  modifier whenSomethingBadHappens() {
    _;
  }

  function testWhenSomethingBadHappens()
    external 
    whenSomethingBadHappens
  {
    // it should not revert
  }

}"
        );

        // Test that "it should revert" actions change the test name.
        let file_contents =
            String::from("file.sol\n└── when something bad happens\n   └── it should revert");

        assert_eq!(
            &scaffold(&file_contents)?,
            r"pragma solidity [VERSION];

contract FileTest {
  modifier whenSomethingBadHappens() {
    _;
  }

  function testRevertsWhenSomethingBadHappens()
    external 
    whenSomethingBadHappens
  {
    // it should revert
  }

}"
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

        assert_eq!(
            &scaffold(&file_contents)?,
            r"pragma solidity [VERSION];

contract Two_childrenTest {
  modifier whenStuffCalled() {
    _;
  }

  modifier whenNotStuffCalled() {
    _;
  }

  function testRevertsWhenStuffCalled()
    external 
    whenStuffCalled
  {
    // it should revert
  }

  function testRevertsWhenNotStuffCalled()
    external 
    whenNotStuffCalled
  {
    // it should revert
  }

}"
        );

        Ok(())
    }

    #[test]
    fn test_action_recollection() -> Result<()> {
        let file_contents = String::from(
            r"actions.sol
└── when stuff called
   ├── it should revert
   ├── it should be cool
   └── it might break
",
        );

        assert_eq!(
            &scaffold(&file_contents)?,
            r"pragma solidity [VERSION];

contract ActionsTest {
  modifier whenStuffCalled() {
    _;
  }

  function testWhenStuffCalled()
    external 
    whenStuffCalled
  {
    // it should revert
    // it should be cool
    // it might break
  }

}"
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

        assert_eq!(
            &scaffold(&file_contents)?,
            r"pragma solidity [VERSION];

contract DeepTest {
  modifier whenStuffCalled() {
    _;
  }

  modifier whenNotStuffCalled() {
    _;
  }

  modifier whenTheDepositAmountIsZero() {
    _;
  }

  modifier whenTheDepositAmountIsNotZero() {
    _;
  }

  modifier whenTheNumberCountIsZero() {
    _;
  }

  modifier whenTheAssetIsNotAContract() {
    _;
  }

  modifier whenTheAssetIsAContract() {
    _;
  }

  modifier whenTheAssetMissesTheERC_20ReturnValue() {
    _;
  }

  modifier whenTheAssetDoesNotMissTheERC_20ReturnValue() {
    _;
  }

  function testRevertsWhenStuffCalled()
    external 
    whenStuffCalled
  {
    // it should revert
  }

  function testRevertsWhenTheDepositAmountIsZero()
    external 
    whenNotStuffCalled
    whenTheDepositAmountIsZero
  {
    // it should revert
  }

  function testRevertsWhenTheNumberCountIsZero()
    external 
    whenNotStuffCalled
    whenTheDepositAmountIsNotZero
    whenTheNumberCountIsZero
  {
    // it should revert
  }

  function testRevertsWhenTheAssetIsNotAContract()
    external 
    whenNotStuffCalled
    whenTheDepositAmountIsNotZero
    whenTheAssetIsNotAContract
  {
    // it should revert
  }

  function testWhenTheAssetMissesTheERC_20ReturnValue()
    external 
    whenNotStuffCalled
    whenTheDepositAmountIsNotZero
    whenTheAssetIsAContract
    whenTheAssetMissesTheERC_20ReturnValue
  {
    // it should create the child
    // it should perform the ERC-20 transfers
    // it should emit a {MultipleChildren} event
  }

  function testWhenTheAssetDoesNotMissTheERC_20ReturnValue()
    external 
    whenNotStuffCalled
    whenTheDepositAmountIsNotZero
    whenTheAssetIsAContract
    whenTheAssetDoesNotMissTheERC_20ReturnValue
  {
    // it should create the child
    // it should emit a {MultipleChildren} event
  }

}"
        );

        Ok(())
    }
}
