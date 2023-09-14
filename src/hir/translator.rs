//! The implementation of a translator between a bulloak tree AST and a
//! high-level intermediate representation (HIR) -- AST -> HIR.

use indexmap::IndexMap;

use crate::hir::{self, Hir};
use crate::syntax::ast;
use crate::syntax::visitor::Visitor;
use crate::utils::{capitalize_first_letter, sanitize};

/// A translator between a bulloak tree abstract syntax tree (AST)
/// and a high-level intermediate representation (HIR) -- AST -> HIR.
///
/// It visits an AST in depth-first order an generates a HIR
/// as a result.
pub struct Translator;

impl Translator {
    /// Create a new translator with the given solidity version.
    pub fn new() -> Self {
        Self {}
    }

    /// Translate an AST to a HIR.
    ///
    /// This function is the entry point of the translator.
    pub fn translate(self, ast: &ast::Ast, modifiers: &IndexMap<String, String>) -> Hir {
        TranslatorI::new(self, modifiers).translate(ast)
    }
}

/// The internal implementation of the Translator.
struct TranslatorI<'a> {
    /// A stack of modifiers that will be applied to the
    /// currently visited function.
    ///
    /// This stack is updated as the translator traverses the AST.
    /// When the translator finishes traversing a condition, it
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
}

impl<'a> TranslatorI<'a> {
    /// Creates a new internal translator.
    fn new(_translator: Translator, modifiers: &'a IndexMap<String, String>) -> Self {
        Self {
            modifier_stack: Vec::new(),
            modifiers,
        }
    }

    /// Concrete implementation of the translation from AST to HIR.
    fn translate(&mut self, ast: &ast::Ast) -> Hir {
        let mut hirs = match ast {
            ast::Ast::Root(ref root) => self.visit_root(root).unwrap(),
            _ => unreachable!(),
        };

        // The result of translating is a Vec<Hir> where the only member
        // is a Root HIR node.
        std::mem::take(&mut hirs[0])
    }
}

impl<'a> Visitor for TranslatorI<'a> {
    type Output = Vec<Hir>;
    type Error = ();

    fn visit_root(&mut self, root: &crate::syntax::ast::Root) -> Result<Self::Output, Self::Error> {
        let mut root_children = Vec::new();

        let mut contract_children = Vec::new();
        for ast in &root.children {
            match ast {
                // A root node cannot be a child of a root node.
                ast::Ast::Root(_) => unreachable!(),
                // Found a top-level action. This corresponds to a function.
                ast::Ast::Action(action) => {
                    let words = action.title.split_whitespace();
                    let words = words.skip(1); // Removes "it" from the test name.

                    // Map an iterator over the words of an action to the test name.
                    //
                    // Example: [do, stuff] -> DoStuff
                    let test_name =
                        words.fold(String::with_capacity(action.title.len()), |mut acc, w| {
                            acc.reserve(w.len() + 1);
                            acc.push_str(&capitalize_first_letter(w));
                            acc
                        });

                    // We need to sanitize here and not in a previous compiler
                    // phase because we want to emit the action as is in a comment.
                    let test_name = sanitize(&test_name);
                    let test_name = format!("test_{}", test_name);

                    let hirs = self.visit_action(action)?;
                    let hir = Hir::FunctionDefinition(hir::FunctionDefinition {
                        identifier: test_name,
                        ty: hir::FunctionTy::Function,
                        modifiers: None,
                        children: Some(hirs),
                    });
                    contract_children.push(hir);
                }
                ast::Ast::Condition(condition) => {
                    contract_children.append(&mut self.visit_condition(condition)?);
                }
            }
        }

        // Add the contract definition to the hir.
        root_children.push(Hir::ContractDefinition(hir::ContractDefinition {
            identifier: root.contract_name.clone(),
            children: contract_children,
        }));

        Ok(vec![Hir::Root(hir::Root {
            children: root_children,
        })])
    }

    fn visit_condition(
        &mut self,
        condition: &crate::syntax::ast::Condition,
    ) -> Result<Self::Output, Self::Error> {
        let mut children = Vec::new();

        // It's fine to unwrap here because we discover all modifiers in a previous pass.
        let modifier = self.modifiers.get(&condition.title).unwrap();
        self.modifier_stack.push(modifier);

        // Add a modifier node.
        let hir = Hir::FunctionDefinition(hir::FunctionDefinition {
            identifier: modifier.clone(),
            ty: hir::FunctionTy::Modifier,
            modifiers: None,
            children: None,
        });
        children.push(hir);

        // We first visit all actions in order to keep the functions
        // in the same order that they appear in the source .tree text.
        let mut actions = Vec::new();
        for action in &condition.children {
            if let ast::Ast::Action(action) = action {
                actions.append(&mut self.visit_action(action)?);
            }
        }

        // Add this condition's function definition if it has children actions.
        if actions.len() > 0 {
            // If the only action is `it should revert`, we slightly change the function name
            // to reflect this.
            let is_revert = actions.len() == 1
                && actions.get(0).is_some_and(|action| {
                    if let hir::Hir::Comment(comment) = action {
                        comment.lexeme == "it should revert"
                    } else {
                        false
                    }
                });

            // It's fine to unwrap here because we check that no action appears outside of a condition.
            let last_modifier = self.modifier_stack.last().unwrap();
            let function_name = if is_revert {
                let mut words = condition.title.split_whitespace();
                // It is fine to unwrap because conditions have at least one word in them.
                let keyword = capitalize_first_letter(words.next().unwrap());

                // Map an iterator over the words of a condition to the test name.
                //
                // Example: [when, something, happens] -> WhenSomethingHappens
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

            let hir = Hir::FunctionDefinition(hir::FunctionDefinition {
                identifier: function_name,
                ty: hir::FunctionTy::Function,
                modifiers: Some(self.modifier_stack.iter().map(|&m| m.to_owned()).collect()),
                children: Some(actions),
            });
            children.push(hir);
        }

        // Then we recursively visit all child conditions.
        for condition in &condition.children {
            if let ast::Ast::Condition(condition) = condition {
                children.append(&mut self.visit_condition(condition)?);
            }
        }

        self.modifier_stack.pop();

        Ok(children)
    }

    fn visit_action(
        &mut self,
        action: &crate::syntax::ast::Action,
    ) -> Result<Self::Output, Self::Error> {
        Ok(vec![hir::Hir::Comment(hir::Comment {
            lexeme: action.title.clone(),
        })])
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use pretty_assertions::assert_eq;

    use crate::hir::{self, Hir};
    use crate::scaffold::modifiers;
    use crate::syntax::parser::Parser;
    use crate::syntax::tokenizer::Tokenizer;

    fn translate(text: &str) -> Result<hir::Hir> {
        let tokens = Tokenizer::new().tokenize(&text)?;
        let ast = Parser::new().parse(&text, &tokens)?;
        let mut discoverer = modifiers::ModifierDiscoverer::new();
        let modifiers = discoverer.discover(&ast);

        Ok(hir::translator::Translator::new().translate(&ast, modifiers))
    }

    fn root(children: Vec<Hir>) -> Hir {
        Hir::Root(hir::Root { children })
    }

    fn contract(identifier: String, children: Vec<Hir>) -> Hir {
        Hir::ContractDefinition(hir::ContractDefinition {
            identifier,
            children,
        })
    }

    fn function(
        identifier: String,
        ty: hir::FunctionTy,
        modifiers: Option<Vec<String>>,
        children: Option<Vec<Hir>>,
    ) -> Hir {
        Hir::FunctionDefinition(hir::FunctionDefinition {
            identifier,
            ty,
            modifiers,
            children,
        })
    }

    fn comment(lexeme: String) -> Hir {
        Hir::Comment(hir::Comment { lexeme })
    }

    #[test]
    fn test_one_child() {
        assert_eq!(
            translate("Foo_Test\n└── when something bad happens\n   └── it should revert").unwrap(),
            root(vec![contract(
                "Foo_Test".to_owned(),
                vec![
                    function(
                        "whenSomethingBadHappens".to_owned(),
                        hir::FunctionTy::Modifier,
                        None,
                        None
                    ),
                    function(
                        "test_RevertWhen_SomethingBadHappens".to_owned(),
                        hir::FunctionTy::Function,
                        Some(vec!["whenSomethingBadHappens".to_owned()]),
                        Some(vec![comment("it should revert".to_owned())])
                    ),
                ]
            )])
        );
    }

    #[test]
    fn test_two_children() {
        assert_eq!(
            translate(
                r"FooBarTheBest_Test
├── when stuff called
│  └── it should revert
└── given not stuff called
   └── it should revert"
            )
            .unwrap(),
            root(vec![contract(
                "FooBarTheBest_Test".to_owned(),
                vec![
                    function(
                        "whenStuffCalled".to_owned(),
                        hir::FunctionTy::Modifier,
                        None,
                        None
                    ),
                    function(
                        "test_RevertWhen_StuffCalled".to_owned(),
                        hir::FunctionTy::Function,
                        Some(vec!["whenStuffCalled".to_owned()]),
                        Some(vec![comment("it should revert".to_owned())])
                    ),
                    function(
                        "givenNotStuffCalled".to_owned(),
                        hir::FunctionTy::Modifier,
                        None,
                        None
                    ),
                    function(
                        "test_RevertGiven_NotStuffCalled".to_owned(),
                        hir::FunctionTy::Function,
                        Some(vec!["givenNotStuffCalled".to_owned()]),
                        Some(vec![comment("it should revert".to_owned())])
                    ),
                ]
            )])
        );
    }

    #[test]
    fn test_action_with_sibling_condition() -> Result<()> {
        let file_contents = String::from(
            r"
Foo_Test
└── when stuff called
    ├── It should do stuff.
    ├── when a called
    │   └── it should revert
    ├── It should do more.
    └── when b called
        └── it should not revert",
        );

        assert_eq!(
            translate(&file_contents)?,
            root(vec![contract(
                "Foo_Test".to_owned(),
                vec![
                    function(
                        "whenStuffCalled".to_owned(),
                        hir::FunctionTy::Modifier,
                        None,
                        None
                    ),
                    function(
                        "test_WhenStuffCalled".to_owned(),
                        hir::FunctionTy::Function,
                        Some(vec!["whenStuffCalled".to_owned()]),
                        Some(vec![
                            comment("It should do stuff.".to_owned()),
                            comment("It should do more.".to_owned())
                        ])
                    ),
                    function(
                        "whenACalled".to_owned(),
                        hir::FunctionTy::Modifier,
                        None,
                        None
                    ),
                    function(
                        "test_RevertWhen_ACalled".to_owned(),
                        hir::FunctionTy::Function,
                        Some(vec!["whenStuffCalled".to_owned(), "whenACalled".to_owned()]),
                        Some(vec![comment("it should revert".to_owned())])
                    ),
                    function(
                        "whenBCalled".to_owned(),
                        hir::FunctionTy::Modifier,
                        None,
                        None
                    ),
                    function(
                        "test_WhenBCalled".to_owned(),
                        hir::FunctionTy::Function,
                        Some(vec!["whenStuffCalled".to_owned(), "whenBCalled".to_owned()]),
                        Some(vec![comment("it should not revert".to_owned())])
                    ),
                ]
            )])
        );

        Ok(())
    }
}
