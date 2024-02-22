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
                    root.children.push(Hir::ContractDefinition(child_contract));
                    contract_definition = match &mut root.children[0] {
                        Hir::ContractDefinition(contract) => contract,
                        _ => unreachable!(),
                    }
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
                            if !added_modifiers.contains(&func_def.identifier) {
                                added_modifiers.insert(func_def.identifier.clone());
                            }
                        };

                        contract_definition.children.push(child.clone());
                    }
                }
            }
        }

        Ok(Hir::Root(root))
    }
}
