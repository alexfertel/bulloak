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
#[derive(Default)]
pub struct Translator;

impl Translator {
    /// Create a new translator.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    /// Translate an AST to a HIR.
    ///
    /// This function is the entry point of the translator.
    #[must_use]
    pub fn translate(&self, ast: &ast::Ast, modifiers: &IndexMap<String, String>, add_vm_skip: &bool) -> Hir {
        TranslatorI::new(modifiers, add_vm_skip).translate(ast)
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
    /// Whether to add `vm.skip(true)` at the beginning of each test.
    add_vm_skip: &'a bool,
}

impl<'a> TranslatorI<'a> {
    /// Creates a new internal translator.
    fn new(modifiers: &'a IndexMap<String, String>, add_vm_skip: &'a bool) -> Self {
        Self {
            modifier_stack: Vec::new(),
            modifiers,
            add_vm_skip
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
                // Root or ActionDescription nodes cannot be children of a root node. This
                // must be handled in a previous pass.
                ast::Ast::Root(_) | ast::Ast::ActionDescription(_) => unreachable!(),
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
                    let test_name = format!("test_{test_name}");

                    let hirs = self.visit_action(action)?;

                    let hir = Hir::FunctionDefinition(hir::FunctionDefinition {
                        identifier: test_name,
                        ty: hir::FunctionTy::Function,
                        span: action.span,
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

        let action_count = condition
            .children
            .iter()
            .filter(|child| ast::Ast::is_action(child))
            .count();
        // If this condition only has actions as children, then we don't generate
        // a modifier for it, since it would only be used in the emitted function.
        if condition.children.len() != action_count {
            if let Some(modifier) = self.modifiers.get(&condition.title) {
                self.modifier_stack.push(modifier);
                // Add a modifier node.
                let hir = Hir::FunctionDefinition(hir::FunctionDefinition {
                    identifier: modifier.clone(),
                    ty: hir::FunctionTy::Modifier,
                    span: condition.span,
                    modifiers: None,
                    children: None,
                });
                children.push(hir);
            };
        }

        // We first visit all actions in order to keep the functions
        // in the same order that they appear in the source .tree text.
        let mut actions = Vec::new();
        for action in &condition.children {
            if let ast::Ast::Action(action) = action {
                actions.append(&mut self.visit_action(action)?);
            }
        }

        // Add this condition's function definition if it has children actions.
        if !actions.is_empty() {
            // If the only action is `it should revert`, we slightly change the function name
            // to reflect this.
            let is_revert = actions.len() == 1
                && actions.first().is_some_and(|action| {
                    if let hir::Hir::Comment(comment) = action {
                        comment.lexeme == "it should revert"
                    } else {
                        false
                    }
                });

            let mut words = condition.title.split_whitespace();
            // It is fine to unwrap because conditions have at least one word in them.
            let keyword = capitalize_first_letter(words.next().unwrap());

            let function_name = if is_revert {
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
                format!("test_Revert{keyword}_{test_name}")
            } else {
                // Map an iterator over the words of a condition to the test name.
                //
                // Example: [when, something, happens] -> WhenSomethingHappens
                let test_name = words.fold(keyword, |mut acc, w| {
                    acc.reserve(w.len() + 1);
                    acc.push_str(&capitalize_first_letter(w));
                    acc
                });

                format!("test_{test_name}")
            };

            let modifiers = if self.modifier_stack.is_empty() {
                None
            } else {
                Some(self.modifier_stack.iter().map(|&m| m.to_owned()).collect())
            };

            if *self.add_vm_skip {
                actions.insert(
                    0,
                    Hir::Expression(hir::Expression {
                        ty: hir::SupportedExpressionType::MemberAccess,
                        expression: "vm.skip(true)".to_string(),
                    }),
                );
            }

            let hir = Hir::FunctionDefinition(hir::FunctionDefinition {
                identifier: function_name,
                ty: hir::FunctionTy::Function,
                span: condition.span,
                modifiers,
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

        if condition.children.len() != action_count {
            self.modifier_stack.pop();
        }

        Ok(children)
    }

    fn visit_action(
        &mut self,
        action: &crate::syntax::ast::Action,
    ) -> Result<Self::Output, Self::Error> {
        let mut descriptions = vec![];
        for description in &action.children {
            if let ast::Ast::ActionDescription(description) = description {
                descriptions.append(&mut self.visit_description(description)?);
            }
        }

        Ok(std::iter::once(hir::Hir::Comment(hir::Comment {
            lexeme: action.title.clone(),
        }))
        .chain(descriptions)
        .collect())
    }

    fn visit_description(
        &mut self,
        description: &crate::syntax::ast::Description,
    ) -> Result<Self::Output, Self::Error> {
        Ok(vec![hir::Hir::Comment(hir::Comment {
            lexeme: description.text.clone(),
        })])
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use pretty_assertions::assert_eq;

    use crate::hir::{self, Hir};
    use crate::scaffold::modifiers;
    use crate::span::{Position, Span};
    use crate::syntax::parser::Parser;
    use crate::syntax::tokenizer::Tokenizer;

    fn translate(text: &str, add_vm_skip: &bool) -> Result<hir::Hir> {
        let tokens = Tokenizer::new().tokenize(&text)?;
        let ast = Parser::new().parse(&text, &tokens)?;
        let mut discoverer = modifiers::ModifierDiscoverer::new();
        let modifiers = discoverer.discover(&ast);

        Ok(hir::translator::Translator::new().translate(&ast, modifiers, add_vm_skip))
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
        span: Span,
        modifiers: Option<Vec<String>>,
        children: Option<Vec<Hir>>,
    ) -> Hir {
        Hir::FunctionDefinition(hir::FunctionDefinition {
            identifier,
            ty,
            span,
            modifiers,
            children,
        })
    }

    fn expression(content: String, ty: hir::SupportedExpressionType) -> Hir {
        Hir::Expression(hir::Expression {
            ty,
            expression: content,
        })
    }

    fn comment(lexeme: String) -> Hir {
        Hir::Comment(hir::Comment { lexeme })
    }

    #[test]
    fn one_child() {
        assert_eq!(
            translate("Foo_Test\n└── when something bad happens\n   └── it should revert", &true).unwrap(),
            root(vec![contract(
                "Foo_Test".to_owned(),
                vec![function(
                    "test_RevertWhen_SomethingBadHappens".to_owned(),
                    hir::FunctionTy::Function,
                    Span::new(Position::new(9, 2, 1), Position::new(74, 3, 23)),
                    None,
                    Some(vec![
                        expression("vm.skip(true)".to_string(), hir::SupportedExpressionType::MemberAccess),
                        comment("it should revert".to_owned())
                    ])
                ),]
            )])
        );
    }

    #[test]
    fn two_children() {
        assert_eq!(
            translate(
                r"FooBarTheBest_Test
├── when stuff called
│  └── it should revert
└── given not stuff called
   └── it should revert", &true
            )
            .unwrap(),
            root(vec![contract(
                "FooBarTheBest_Test".to_owned(),
                vec![
                    function(
                        "test_RevertWhen_StuffCalled".to_owned(),
                        hir::FunctionTy::Function,
                        Span::new(Position::new(19, 2, 1), Position::new(77, 3, 23)),
                        None,
                        Some(vec![
                            expression("vm.skip(true)".to_string(), hir::SupportedExpressionType::MemberAccess),
                            comment("it should revert".to_owned())
                        ])
                    ),
                    function(
                        "test_RevertGiven_NotStuffCalled".to_owned(),
                        hir::FunctionTy::Function,
                        Span::new(Position::new(79, 4, 1), Position::new(140, 5, 23)),
                        None,
                        Some(vec![
                            expression("vm.skip(true)".to_string(), hir::SupportedExpressionType::MemberAccess),
                            comment("it should revert".to_owned())
                        ])
                    ),
                ]
            )])
        );
    }

    #[test]
    fn action_with_sibling_condition() -> Result<()> {
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
            translate(&file_contents, &true)?,
            root(vec![contract(
                "Foo_Test".to_owned(),
                vec![
                    function(
                        "whenStuffCalled".to_owned(),
                        hir::FunctionTy::Modifier,
                        Span::new(Position::new(10, 3, 1), Position::new(235, 9, 32)),
                        None,
                        None
                    ),
                    function(
                        "test_WhenStuffCalled".to_owned(),
                        hir::FunctionTy::Function,
                        Span::new(Position::new(10, 3, 1), Position::new(235, 9, 32)),
                        Some(vec!["whenStuffCalled".to_owned()]),
                        Some(vec![
                            expression("vm.skip(true)".to_string(), hir::SupportedExpressionType::MemberAccess),
                            comment("It should do stuff.".to_owned()),
                            comment("It should do more.".to_owned())
                        ])
                    ),
                    function(
                        "test_RevertWhen_ACalled".to_owned(),
                        hir::FunctionTy::Function,
                        Span::new(Position::new(76, 5, 5), Position::new(135, 6, 28)),
                        Some(vec!["whenStuffCalled".to_owned()]),
                        Some(vec![
                            expression("vm.skip(true)".to_string(), hir::SupportedExpressionType::MemberAccess),
                            comment("it should revert".to_owned())
                        ])
                    ),
                    function(
                        "test_WhenBCalled".to_owned(),
                        hir::FunctionTy::Function,
                        Span::new(Position::new(174, 8, 5), Position::new(235, 9, 32)),
                        Some(vec!["whenStuffCalled".to_owned()]),
                        Some(vec![
                            expression("vm.skip(true)".to_string(), hir::SupportedExpressionType::MemberAccess),
                            comment("it should not revert".to_owned())
                        ])
                    ),
                ]
            )])
        );

        Ok(())
    }
}
