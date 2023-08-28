use indexmap::IndexMap;
use std::result;

use crate::{
    ast::{self, Ast},
    utils::{capitalize_first_letter, sanitize},
    visitor::Visitor,
};

/// Solidity code emitter.
///
/// This struct holds the state of the emitter. It is not
/// tied to a specific AST.
pub struct Emitter<'s> {
    /// This flag determines whether actions are emitted as comments
    /// in the body of functions.
    with_actions_as_comments: bool,
    /// The indentation level of the emitted code.
    indent: usize,
    /// The solidity version to use in the emitted code.
    solidity_version: &'s str,
}

impl<'s> Emitter<'s> {
    /// Create a new emitter with the given configuration.
    pub fn new(with_actions_as_comments: bool, indent: usize, solidity_version: &'s str) -> Self {
        Self {
            with_actions_as_comments,
            indent,
            solidity_version,
        }
    }

    /// Emit Solidity code from the given AST.
    pub fn emit(self, ast: &ast::Ast, modifiers: &IndexMap<String, String>) -> String {
        EmitterI::new(self, modifiers).emit(ast)
    }

    /// Return the indentation string. i.e. the string that is used
    /// to indent the emitted code.
    fn indent(&self) -> String {
        " ".repeat(self.indent)
    }
}

/// The internal implementation of the solidity code emitter.
///
/// This emitter generates skeleton contracts and tests functions
/// inside that contract described in the input .tree file.
struct EmitterI<'a> {
    /// A stack of modifiers that will be applied to the
    /// currently emitted function.
    ///
    /// This stack is updated as the emitter traverses the AST.
    /// When the emitter finishes traversing a condition, it
    /// pops the last modifier from the stack, since it won't
    /// be applied to the next function. The rest of the modifiers
    /// might be applied in case there are more sibling actions or
    /// conditions.
    modifier_stack: Vec<&'a str>,
    /// A map of condition titles to their corresponding modifiers.
    ///
    /// This map is used to retrieve a modifier given a condition title
    /// to improve performance. Otherwise each title would be converted
    /// to a modifier every time it is used.
    modifiers: &'a IndexMap<String, String>,
    /// The emitter state.
    emitter: Emitter<'a>,
}

impl<'a> EmitterI<'a> {
    /// Create a new emitter with the given emitter state and modifier map.
    fn new(emitter: Emitter<'a>, modifiers: &'a IndexMap<String, String>) -> Self {
        Self {
            modifier_stack: Vec::new(),
            modifiers,
            emitter,
        }
    }

    /// Emit Solidity code from the given AST.
    ///
    /// This function is the entry point of the emitter.
    fn emit(&mut self, ast: &ast::Ast) -> String {
        match ast {
            Ast::Root(ref root) => self.visit_root(root).unwrap(),
            _ => unreachable!(),
        }
    }

    /// Emit the contract's definition header.
    ///
    /// This includes:
    /// - The Solidity version pragma.
    /// - The contract's name.
    fn emit_contract_header(&self, root: &ast::Root) -> String {
        let mut emitted = String::new();
        emitted.push_str(&format!(
            "pragma solidity {};\n\n",
            self.emitter.solidity_version
        ));

        // It's fine to unwrap here because we check that the filename always has an extension.
        let contract_name = root.file_name.split_once('.').unwrap().0;
        let contract_name = sanitize(&capitalize_first_letter(contract_name));
        emitted.push_str(format!("contract {}Test {{\n", contract_name).as_str());

        emitted
    }

    /// Emit a modifier.
    ///
    /// A modifier follows the following structure:
    /// ```solidity
    /// modifier [MODIFIER_NAME]() {
    ///    _;
    /// }
    /// ```
    fn emit_modifier(&self, modifier: &str) -> String {
        let mut emitted = String::new();
        let indentation = self.emitter.indent();
        emitted.push_str(&format!("{}modifier {}() {{\n", indentation, modifier));
        emitted.push_str(&format!("{}_;\n", indentation.repeat(2)));
        emitted.push_str(&format!("{}}}\n", indentation));
        emitted.push('\n');

        emitted
    }

    /// Emit a function's definition header.
    ///
    /// This includes:
    /// - The function's name.
    /// - The function's visibility.
    /// - Any modifiers that should be applied to the function.
    fn emit_fn_header(&self, condition: &ast::Condition) -> String {
        let mut emitted = String::new();

        // We count instead of collecting into a Vec to avoid allocating a Vec for each condition.
        let action_count = condition.asts.iter().filter(|ast| ast.is_action()).count();
        let mut actions = condition.asts.iter().filter(|ast| ast.is_action());

        if action_count > 0 {
            let fn_indentation = self.emitter.indent();
            let fn_body_indentation = fn_indentation.repeat(2);

            // If the only action is `it should revert`, we slightly change the function name
            // to reflect this.
            let is_revert = action_count == 1
                && actions.next().is_some_and(|action| {
                    if let Ast::Action(action) = action {
                        action.title == "it should revert"
                    } else {
                        false
                    }
                });

            // It's fine to unwrap here because we check that no action appears outside of a condition.
            let last_modifier = self.modifier_stack.last().unwrap();
            let function_name = if is_revert {
                let mut words = condition.title.split(' ');
                // It is fine to unwrap because conditions have at least one word in them.
                let keyword = capitalize_first_letter(words.next().unwrap());
                let test_name = words.fold(
                    String::with_capacity(condition.title.len() - keyword.len()),
                    |mut acc, w| {
                        acc.reserve(w.len() + 1);
                        acc.push_str(&capitalize_first_letter(w));
                        acc
                    },
                );

                // The structure for a function name when it is a revert is:
                //
                // test_Revert[KEYWORD]_Description
                //
                // where `KEYWORD` is the starting word of the condition.
                format!("test_Revert{}_{}", keyword, test_name)
            } else {
                let test_name = capitalize_first_letter(last_modifier);
                format!("test_{}", test_name)
            };

            emitted.push_str(format!("{}function {}()\n", fn_indentation, function_name).as_str());
            emitted.push_str(format!("{}external\n", fn_body_indentation).as_str());

            // Emit the modifiers that should be applied to this function.
            for modifier in &self.modifier_stack {
                emitted.push_str(format!("{}{}\n", fn_body_indentation, modifier).as_str());
            }

            emitted.push_str(format!("{}{{\n", fn_indentation).as_str());
        }

        emitted
    }
}

/// The visitor implementation for the emitter.
///
/// Note that the visitor is infallible because previous
/// passes ensure that the AST is valid. In case an error
/// is found, it should be added to a previous pass.
impl<'a> Visitor for EmitterI<'a> {
    type Output = String;
    type Error = ();

    fn visit_root(&mut self, root: &ast::Root) -> result::Result<Self::Output, Self::Error> {
        let mut emitted = String::new();

        let contract_header = self.emit_contract_header(root);
        emitted.push_str(&contract_header);

        for condition in &root.asts {
            if let Ast::Condition(condition) = condition {
                emitted.push_str(&self.visit_condition(condition)?);
            }
        }

        // Remove the last char, which is the extra '\n' from
        // emitting functions.
        emitted.pop();
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

        emitted.push_str(&self.emit_modifier(modifier));

        let fn_header = self.emit_fn_header(condition);
        emitted.push_str(&fn_header);

        // We first visit all actions in order to emit the functions
        // in the same order as they appear in the source .tree text.
        for action in &condition.asts {
            if let Ast::Action(action) = action {
                emitted.push_str(&self.visit_action(action)?);
            }
        }

        // Then we recursively emit all child conditions.
        for condition in &condition.asts {
            if let Ast::Condition(condition) = condition {
                emitted.push_str(&self.visit_condition(condition)?);
            }
        }

        // We count instead of collecting into a Vec to avoid allocating a Vec for each condition.
        let action_count = condition.asts.iter().filter(|ast| ast.is_action()).count();
        // We check that there is more than one action to avoid printing extra closing
        // braces when conditions are nested.
        if action_count > 0 {
            emitted.push_str(format!("{}}}\n\n", self.emitter.indent()).as_str());
        }

        self.modifier_stack.pop();

        Ok(emitted)
    }

    fn visit_action(&mut self, action: &ast::Action) -> result::Result<Self::Output, Self::Error> {
        let mut emitted = String::new();

        if self.emitter.with_actions_as_comments {
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

    fn scaffold_with_flags(
        text: &str,
        with_comments: bool,
        indent: usize,
        version: &str,
    ) -> Result<String> {
        let tokens = Tokenizer::new().tokenize(&text)?;
        let ast = Parser::new().parse(&text, &tokens)?;
        let mut discoverer = modifiers::ModifierDiscoverer::new();
        let modifiers = discoverer.discover(&ast);
        Ok(emitter::Emitter::new(with_comments, indent, version).emit(&ast, &modifiers))
    }

    fn scaffold(text: &str) -> Result<String> {
        scaffold_with_flags(text, true, 2, "0.8.0")
    }

    #[test]
    fn test_one_child() -> Result<()> {
        let file_contents =
            String::from("file.sol\n└── when something bad happens\n   └── it should not revert");

        assert_eq!(
            &scaffold(&file_contents)?,
            r"pragma solidity 0.8.0;

contract FileTest {
  modifier whenSomethingBadHappens() {
    _;
  }

  function test_WhenSomethingBadHappens()
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
            r"pragma solidity 0.8.0;

contract FileTest {
  modifier whenSomethingBadHappens() {
    _;
  }

  function test_RevertWhen_SomethingBadHappens()
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
    fn test_without_actions_as_comments() -> Result<()> {
        let file_contents =
            String::from("file.sol\n└── when something bad happens\n   └── it should not revert");

        assert_eq!(
            &scaffold_with_flags(&file_contents, false, 2, "0.8.0")?,
            r"pragma solidity 0.8.0;

contract FileTest {
  modifier whenSomethingBadHappens() {
    _;
  }

  function test_WhenSomethingBadHappens()
    external
    whenSomethingBadHappens
  {
  }
}"
        );

        Ok(())
    }

    #[test]
    fn test_unsanitized_input() -> Result<()> {
        let file_contents =
            String::from("fi-e.sol\n└── when something bad happens\n   └── it should not revert");

        assert_eq!(
            &scaffold_with_flags(&file_contents, false, 2, "0.8.0")?,
            r"pragma solidity 0.8.0;

contract Fi_eTest {
  modifier whenSomethingBadHappens() {
    _;
  }

  function test_WhenSomethingBadHappens()
    external
    whenSomethingBadHappens
  {
  }
}"
        );

        Ok(())
    }

    #[test]
    fn test_indentation() -> Result<()> {
        let file_contents =
            String::from("file.sol\n└── when something bad happens\n   └── it should not revert");

        assert_eq!(
            &scaffold_with_flags(&file_contents, false, 4, "0.8.0")?,
            r"pragma solidity 0.8.0;

contract FileTest {
    modifier whenSomethingBadHappens() {
        _;
    }

    function test_WhenSomethingBadHappens()
        external
        whenSomethingBadHappens
    {
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
            r"pragma solidity 0.8.0;

contract Two_childrenTest {
  modifier whenStuffCalled() {
    _;
  }

  function test_RevertWhen_StuffCalled()
    external
    whenStuffCalled
  {
    // it should revert
  }

  modifier whenNotStuffCalled() {
    _;
  }

  function test_RevertWhen_NotStuffCalled()
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
            r"pragma solidity 0.8.0;

contract ActionsTest {
  modifier whenStuffCalled() {
    _;
  }

  function test_WhenStuffCalled()
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
      └── given the asset is a contract
          ├── when the asset misses the ERC-20 return value
          │  ├── it should create the child
          │  ├── it should perform the ERC-20 transfers
          │  └── it should emit a {MultipleChildren} event
          └── when the asset does not miss the ERC-20 return value
              ├── it should create the child
              └── it should emit a {MultipleChildren} event"#,
        );

        assert_eq!(
            &scaffold(&file_contents)?,
            r"pragma solidity 0.8.0;

contract DeepTest {
  modifier whenStuffCalled() {
    _;
  }

  function test_RevertWhen_StuffCalled()
    external
    whenStuffCalled
  {
    // it should revert
  }

  modifier whenNotStuffCalled() {
    _;
  }

  modifier whenTheDepositAmountIsZero() {
    _;
  }

  function test_RevertWhen_TheDepositAmountIsZero()
    external
    whenNotStuffCalled
    whenTheDepositAmountIsZero
  {
    // it should revert
  }

  modifier whenTheDepositAmountIsNotZero() {
    _;
  }

  modifier whenTheNumberCountIsZero() {
    _;
  }

  function test_RevertWhen_TheNumberCountIsZero()
    external
    whenNotStuffCalled
    whenTheDepositAmountIsNotZero
    whenTheNumberCountIsZero
  {
    // it should revert
  }

  modifier whenTheAssetIsNotAContract() {
    _;
  }

  function test_RevertWhen_TheAssetIsNotAContract()
    external
    whenNotStuffCalled
    whenTheDepositAmountIsNotZero
    whenTheAssetIsNotAContract
  {
    // it should revert
  }

  modifier givenTheAssetIsAContract() {
    _;
  }

  modifier whenTheAssetMissesTheERC_20ReturnValue() {
    _;
  }

  function test_WhenTheAssetMissesTheERC_20ReturnValue()
    external
    whenNotStuffCalled
    whenTheDepositAmountIsNotZero
    givenTheAssetIsAContract
    whenTheAssetMissesTheERC_20ReturnValue
  {
    // it should create the child
    // it should perform the ERC-20 transfers
    // it should emit a {MultipleChildren} event
  }

  modifier whenTheAssetDoesNotMissTheERC_20ReturnValue() {
    _;
  }

  function test_WhenTheAssetDoesNotMissTheERC_20ReturnValue()
    external
    whenNotStuffCalled
    whenTheDepositAmountIsNotZero
    givenTheAssetIsAContract
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
