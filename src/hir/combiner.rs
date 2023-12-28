//! The implementation of a high-level intermediate representation (HIR) combiner.

use std::{collections::HashSet, fmt, result};

use crate::{utils::get_contract_name_from_identifier, span::Span};

use super::{Hir, Root, ContractDefinition};

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
            ContractNameMismatch(actual, expected) => write!(f, "Contract name mismatch: expected '{expected}', found '{actual}'"),
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
        Error {
            kind,
            text,
            span,
        }
    }

    /// Combines the translated HIRs into a single HIR. HIRs are merged by
    /// iterating over each HIR and merging their children into the contract
    /// definition of the first HIR, while verifying the contract identifiers
    /// match and filtering out duplicate modifiers.
    ///
    /// This function is called after the ASTs are translated to HIR.
    pub fn combine(&self, hirs: &Vec<Hir>) -> Result<Hir> {
        let mut root: Root = Root::default();
        let mut contract_definition = &mut ContractDefinition::default();
        let mut added_modifiers = HashSet::new();

        for hir in hirs {
            match hir {
                Hir::Root(r) => {
                    for child in &r.children {
                        match child {
                            // check the ith HIR's identifier matches the accumulated ContractDefinition identifier
                            // all the ContractDefinitions should be merged into a single child ContractDefinition with the same identifier
                            Hir::ContractDefinition(contract_def) => {
                                if contract_definition.identifier.is_empty() {
                                    let mut child_contract = contract_def.clone();
                                    child_contract.identifier = get_contract_name_from_identifier(&contract_def.identifier);
                                    root.children.push(Hir::ContractDefinition(child_contract));
                                    contract_definition = match &mut root.children[0] {
                                        Hir::ContractDefinition(contract) => contract,
                                        _ => unreachable!(),
                                    }
                                } else {
                                    let identifier = get_contract_name_from_identifier(&contract_def.identifier);
                                    let accumulated_identifier = contract_definition.identifier.clone();
                                    if identifier != accumulated_identifier {
                                        let (text, span) = (String::new(), Span::default()); // @follow-up - how can we get the text and span from the HIR? Is it even necessary? This would be easier to do with verification of the AST. One option is to use the index of the HIR in the vector of HIRs since we know the identifier is the start of a give tree.
                                        Err(self.error(text, span, ErrorKind::ContractNameMismatch(
                                            identifier, accumulated_identifier)
                                        ))?
                                    }
                                    for child in &contract_def.children {
                                        // If child is of type FunctionDefinition with the same identifier as a child of another ContractDefinition of ty
                                        // Modifier, then they are duplicates. Traverse all children of the ContractDefinition and remove the duplicates.
                                        match child {
                                            Hir::FunctionDefinition(func_def) => {
                                                match func_def.ty {
                                                    super::FunctionTy::Modifier => {
                                                        if added_modifiers.contains(&func_def.identifier) {
                                                            // skip this child if the modifier has already been added
                                                            continue;
                                                        } else {
                                                            added_modifiers.insert(func_def.identifier.clone());
                                                        }
                                                    }
                                                    _ => (),
                                                }
                                                contract_definition.children.push(child.clone());
                                            }
                                            // If the child is of type Comment then don't push it to the ContractDefinition
                                            _ => {},
                                        }
                                    }
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                _ => unreachable!(),
            }
        }

        Ok(Hir::Root(root))
    }
}
