//! Defines structural matching as a rule.
//!
//! This rule enforces the following:
//! - All spec-generated functions & modifiers are present in the output file.
//! - The order of the spec-generated functions & modifiers matches the output file.
//!
//! Matching is name-based, which means that two functions are considered the
//! same if:
//! - Their name is exactly the same.
//! - Their function type is exactly the same. Currently, only regular functions
//! and modifiers are supported.

use std::collections::VecDeque;

use solang_parser::pt;

use crate::{
    check::violation::{Violation, ViolationKind},
    hir::{self, Hir},
};

use super::{Checker, Context};

/// An implementation of a structural matching rule.
///
/// Read more at the module-level documentation.
///
/// TODO: Add link to module-level documentation.
pub struct StructuralMatcher;

impl Checker for StructuralMatcher {
    fn check(ctx: &Context) -> anyhow::Result<Vec<Violation>> {
        let mut violations = vec![];

        let hir = ctx.tree_hir;
        // We currently only support one tree per .tree file, which
        // means that there can only be one contract. This is reflected
        // in the current tree hierarchy of the HIR.
        let contract_hir = if let Hir::Root(root) = hir {
            root.children
                .iter()
                .find(|&child| matches!(child, Hir::ContractDefinition(_)))
        } else {
            None
        };

        let pt = &ctx.sol_ast.0;
        // Find the first occurrence of a contract.
        let contract_sol = pt
            .iter()
            .find(|&part| matches!(part, pt::SourceUnitPart::ContractDefinition(_)));
        // If we find no contract, then we check if there is no contract
        // in the HIR, else we found a violation.
        if contract_sol.is_none() {
            if let Some(Hir::ContractDefinition(contract)) = contract_hir {
                let violation =
                    Violation::new(ViolationKind::ContractMissing(contract.identifier.clone()));
                violations.push(violation);

                return Ok(violations);
            };

            // Both contracts are missing, so we're done.
            return Ok(violations);
        }

        // We know a contract exists in both trees because of the
        // check above.
        let contract_hir = contract_hir.unwrap();
        let contract_sol = contract_sol.unwrap();

        // Check that all the functions are present in the
        // output file.
        if let Hir::ContractDefinition(contract_hir) = contract_hir {
            // Check that contract names match.
            if let pt::SourceUnitPart::ContractDefinition(contract_sol) = contract_sol {
                // We won't deal right now with a parsing error from solang_parser.
                if let Some(ref identifier) = contract_sol.name {
                    if identifier.name != contract_hir.identifier {
                        let violation = Violation::new(ViolationKind::ContractNameNotMatches(
                            identifier.name.clone(),
                            contract_hir.identifier.clone(),
                        ));
                        violations.push(violation);
                    }
                };

                violations.append(&mut check_function_structure(contract_hir, contract_sol)?);
            };
        };

        Ok(violations)
    }
}

// Checks that function structures match between the HIR and the solidity AST.
fn check_function_structure(
    contract_hir: &hir::ContractDefinition,
    contract_sol: &pt::ContractDefinition,
) -> anyhow::Result<Vec<Violation>> {
    let mut violations = Vec::new();

    let mut cursor = 0;
    let mut functions_sol = contract_sol.parts.clone();
    for fn_hir in &contract_hir.children {
        if let Hir::FunctionDefinition(fn_hir) = fn_hir {
            let fn_sol = functions_sol.get(cursor);

            if let Some(pt::ContractPart::FunctionDefinition(fn_sol)) = fn_sol {
                if let Some(pt::Identifier { ref name, .. }) = fn_sol.name {
                    if name != &fn_hir.identifier || !fn_types_match(&fn_hir.ty, &fn_sol.ty) {
                        let violation = Violation::new(ViolationKind::TestOrderMismatch(
                            fn_hir.identifier.clone(),
                        ));
                        violations.push(violation);
                    }
                }
            }

            cursor = cursor + 1;
        }
    }

    Ok(violations)
}

/// Checks that the function types between a HIR function
/// and a solang_parser function match.
///
/// We check that the function types match, even though we know that the
/// name not matching is enough, since a modifier will never be
/// named the same as a function per Foundry's best practices.
fn fn_types_match(ty_hir: &hir::FunctionTy, ty_sol: &pt::FunctionTy) -> bool {
    match ty_hir {
        hir::FunctionTy::Function => matches!(ty_sol, pt::FunctionTy::Function),
        hir::FunctionTy::Modifier => matches!(ty_sol, pt::FunctionTy::Modifier),
    }
}
