//! The implementation of a high-level intermediate representation (HIR) combiner.

use std::collections::HashSet;

use crate::utils::get_contract_name_from_identifier;

use super::{Hir, Root, ContractDefinition};

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

    /// Combines the translated HIRs into a single HIR. HIRs are merged by
    /// iterating over each HIR and merging their children into the contract
    /// definition of the first HIR, while verifying the contract identifiers
    /// match and filtering out duplicate modifiers.
    ///
    /// This function is called after the ASTs are translated to HIR.
    pub fn combine(&self, hirs: &Vec<Hir>) -> Option<Hir> {
        let mut root: Root = Root::default();
        let mut contract_definition: &ContractDefinition;
        let mut added_modifiers = HashSet::new();

        for hir in hirs {
            match hir {
                Hir::Root(r) => {
                    for child in &r.children {
                        match child {
                            // check the ith HIR's identifier matches the accumulated ContractDefinition identifier
                            // all the ContractDefinitions should be merged into a single child ContractDefinition with the same identifier
                            Hir::ContractDefinition(contract_def) => {
                                if root.children.is_empty() {
                                    let mut child_contract = contract_def.clone();
                                    child_contract.identifier = get_contract_name_from_identifier(&contract_def.identifier);
                                    root.children.push(Hir::ContractDefinition(child_contract));
                                    contract_definition = &child_contract;
                                } else {
                                    if let identifier = get_contract_name_from_identifier(&contract_def.identifier) != contract_definition.identifier {
                                        Some(format!(
                                            "Contract name mismatch: expected '{}', found '{}'",
                                            identifier, contract_definition.identifier
                                        ));
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

        Some(Hir::Root(root))
    }
}
