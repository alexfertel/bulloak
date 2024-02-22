//! The implementation of a high-level intermediate representation (HIR) combiner.

use std::{collections::HashSet, fmt, result};

use crate::{span::Span, utils::get_contract_name_from_identifier};

use super::{ContractDefinition, FunctionTy, Hir, Root};

type Result<T> = result::Result<T, Error>;

/// An error that occurred while combining HIRs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    /// The kind of error.
    kind: ErrorKind,
    /// The original text that the parser generated the error from. Every
    /// span in an error is a valid range into this string.
    text: String,
    /// The span of this error.
    span: Span,
}

impl std::error::Error for Error {}

impl Error {
    /// Return the type of this error.
    #[must_use]
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// The original text string in which this error occurred.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Return the span at which this error occurred.
    #[must_use]
    pub fn span(&self) -> &Span {
        &self.span
    }
}

type Identifier = String;

/// The type of an error that occurred while combining HIRs.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ErrorKind {
    /// This happens when the contract name in the identifier of one HIR does
    /// not match the contract name in the identifier of another HIR.
    ContractNameMismatch(Identifier, Identifier),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        crate::error::Formatter::from(self).fmt(f)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::ErrorKind::ContractNameMismatch;
        match self {
            ContractNameMismatch(actual, expected) => write!(
                f,
                "contract name mismatch: expected '{expected}', found '{actual}'"
            ),
        }
    }
}

/// A high-level intermediate representation (HIR) combiner.
///
/// It takes a vector of HIRs and combines them into a single HIR
/// by appending the function nodes to the root contract node.
pub struct Combiner;

impl Combiner {
    /// Creates a new combiner.
    #[must_use]
    pub fn new() -> Self {
        Combiner {}
    }

    /// Create a new error with the given span and error type.
    fn error(&self, text: String, span: Span, kind: ErrorKind) -> Error {
        Error { kind, text, span }
    }

    /// Combines the translated HIRs into a single HIR. HIRs are merged by
    /// iterating over each HIR and merging their children into the contract
    /// definition of the first HIR, while verifying the contract identifiers
    /// match and filtering out duplicate modifiers.
    pub fn combine(&self, hirs: &Vec<Hir>) -> Result<Hir> {
        let mut root = Root::default();
        let mut contract_definition = &mut ContractDefinition::default();
        let mut added_modifiers = HashSet::new();

        for hir in hirs {
            let Hir::Root(r) = hir else {
                unreachable!();
            };

            for child in &r.children {
                let Hir::ContractDefinition(contract_def) = child else {
                    // For now we ignore everything that isn't a contract.
                    continue;
                };

                // Check the ith HIR's identifier matches the accumulated ContractDefinition identifier
                // all the ContractDefinitions should be merged into a single child ContractDefinition with the same identifier
                if contract_definition.identifier.is_empty() {
                    let mut child_contract = contract_def.clone();
                    let text = contract_def.identifier.clone();
                    let identifier = get_contract_name_from_identifier(&text)
                        .expect("expected contract identifier at tree root");
                    child_contract.identifier = identifier;

                    // Add modifiers to the list of added modifiers
                    for child in &child_contract.children {
                        let Hir::FunctionDefinition(func_def) = child else {
                            continue;
                        };

                        if let FunctionTy::Modifier = func_def.ty {
                            added_modifiers.insert(func_def.identifier.clone());
                        }
                    }

                    root.children.push(Hir::ContractDefinition(child_contract));
                    contract_definition = match &mut root.children[0] {
                        Hir::ContractDefinition(contract) => contract,
                        _ => unreachable!(),
                    };
                } else {
                    let text = contract_def.identifier.clone();
                    let identifier = get_contract_name_from_identifier(&text).unwrap_or_default();
                    let accumulated_identifier = contract_definition.identifier.clone();
                    if identifier != accumulated_identifier {
                        Err(self.error(
                            text,
                            Span::default(),
                            ErrorKind::ContractNameMismatch(identifier, accumulated_identifier),
                        ))?
                    }

                    for child in &contract_def.children {
                        // If the child isn't a function then don't push it to the ContractDefinition.
                        let Hir::FunctionDefinition(func_def) = child else {
                            continue;
                        };

                        if let FunctionTy::Modifier = func_def.ty {
                            // If child is of type FunctionDefinition with the same identifier as a child of another ContractDefinition of ty
                            // Modifier, then they are duplicates. Traverse all children of the ContractDefinition and remove the duplicates.
                            if added_modifiers.contains(&func_def.identifier) {
                                continue;
                            }
                            added_modifiers.insert(func_def.identifier.clone());
                        };

                        contract_definition.children.push(child.clone());
                    }
                }
            }
        }

        Ok(Hir::Root(root))
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{Error, Result};
    use pretty_assertions::assert_eq;
    use std::panic::catch_unwind;

    use crate::hir::{self, Hir};
    use crate::scaffold::modifiers;
    use crate::span::{Position, Span};
    use crate::syntax::parser::Parser;
    use crate::syntax::tokenizer::Tokenizer;

    fn translate(text: &str) -> Result<Hir> {
        let tokens = Tokenizer::new().tokenize(&text)?;
        let ast = Parser::new().parse(&text, &tokens)?;
        let mut discoverer = modifiers::ModifierDiscoverer::new();
        let modifiers = discoverer.discover(&ast);

        Ok(hir::translator::Translator::new().translate(&ast, modifiers))
    }

    fn combine(hirs: &Vec<Hir>) -> Result<Hir, Error> {
        Ok(crate::hir::combiner::Combiner::new().combine(&hirs)?)
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

    fn comment(lexeme: String) -> Hir {
        Hir::Comment(hir::Comment { lexeme })
    }

    #[test]
    fn panics_when_root_contract_identifier_is_missing() -> Result<()> {
        let trees = vec![
            "::orphanedFunction\n└── when something bad happens\n   └── it should revert",
            "Contract::function\n└── when something bad happens\n   └── it should revert",
        ];
        let hirs = trees
            .iter()
            .map(|tree| translate(tree))
            .collect::<Result<Vec<Hir>>>()?;

        let result = catch_unwind(|| combine(&hirs));
        assert!(result.is_err());
        assert_eq!(
            result
                .unwrap_err()
                .downcast_ref::<String>()
                .unwrap()
                .as_str(),
            "expected contract identifier at tree root"
        );

        Ok(())
    }

    #[test]
    fn errors_when_contract_names_mismatch() -> Result<()> {
        let trees = vec![
            "Contract::function\n└── when something bad happens\n   └── it should revert",
            "::orphanedFunction\n└── when something bad happens\n   └── it should revert",
        ];
        let hirs = trees
            .iter()
            .map(|tree| translate(tree))
            .collect::<Result<Vec<Hir>>>()?;

        let expected = r"•••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••
bulloak error: contract name mismatch: expected 'Contract', found ''";

        match combine(&hirs) {
            Err(e) => assert_eq!(e.to_string(), expected),
            _ => unreachable!("expected an error"),
        }

        Ok(())
    }

    #[test]
    fn duplicate_and_non_function_type_children() -> Result<()> {
        // non-function children aren't pushed to the ContractDefinition
        // duplicate modifiers are deduplicated

        let trees = vec![
            "Contract::function1\n└── when something bad happens\n    └── given something else happens\n   └── it should revert",
            "Contract::function2\n└── when something bad happens\n    └── given the caller is 0x1337\n   └── it should revert",
        ];
        let mut hirs = trees
            .iter()
            .map(|tree| translate(tree))
            .collect::<Result<Vec<Hir>>>()?;

        // append a comment HIR to the hirs
        hirs.push(root(vec![comment("this is a random comment".to_owned())]));

        let children = match combine(&hirs)? {
            Hir::Root(root) => root.children,
            _ => unreachable!(),
        };

        assert_eq!(
            children,
            vec![contract(
                "Contract".to_owned(),
                vec![
                    function(
                        "whenSomethingBadHappens".to_owned(),
                        hir::FunctionTy::Modifier,
                        Span::new(Position::new(20, 2, 1), Position::new(128, 4, 23)),
                        None,
                        None
                    ),
                    function(
                        "test_RevertWhen_SomethingBadHappens".to_owned(),
                        hir::FunctionTy::Function,
                        Span::new(Position::new(20, 2, 1), Position::new(128, 4, 23)),
                        Some(vec!["whenSomethingBadHappens".to_owned()]),
                        Some(vec![comment("it should revert".to_owned())])
                    ),
                    function(
                        "test_RevertWhen_SomethingBadHappens".to_owned(),
                        hir::FunctionTy::Function,
                        Span::new(Position::new(20, 2, 1), Position::new(126, 4, 23)),
                        Some(vec!["whenSomethingBadHappens".to_owned()]),
                        Some(vec![comment("it should revert".to_owned())])
                    ),
                ]
            )]
        );

        Ok(())
    }
}
