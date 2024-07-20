//! Defines structural matching as a rule.
//!
//! This rule enforces the following:
//! - All spec-generated functions & modifiers are present in the output file.
//! - The order of the spec-generated functions & modifiers matches the output
//!   file.
//!
//! Matching is name-based, which means that two functions are considered the
//! same if:
//! - Their name is exactly the same.
//! - Their function type is exactly the same. Currently, only regular functions
//! and modifiers are supported.

use std::collections::BTreeSet;

use bulloak_syntax::utils::sanitize;
use solang_parser::pt;

use super::{Checker, Context};
use crate::{
    check::{
        location::Location,
        utils::offset_to_line,
        violation::{Violation, ViolationKind},
    },
    hir::{self, Hir},
    sol::{find_contract, find_matching_fn},
};

/// An implementation of a structural matching rule.
///
/// Read more at the [module-level documentation].
///
/// module-level documentation: self
pub struct StructuralMatcher;

impl Checker for StructuralMatcher {
    fn check(ctx: &Context) -> Vec<Violation> {
        let mut violations = vec![];

        // We support multiple trees per .tree file, but they are combined into
        // a single HIR during the hir::translate step when creating the context
        // which means that there can only be one contract. This is reflected
        // in the current tree hierarchy of the HIR.
        let contract_hir = match ctx.hir {
            Hir::Root(ref root) => root
                .children
                .iter()
                .find(|&child| matches!(child, Hir::ContractDefinition(_))),
            _ => None,
        };

        // Find the first occurrence of a contract.
        let contract_sol = find_contract(&ctx.pt);
        // If we find no contract in the Solidity file, then there must
        // be no contract in the HIR, else we found a violation.
        if contract_sol.is_none() {
            if let Some(Hir::ContractDefinition(contract)) = contract_hir {
                let violation = Violation::new(
                    ViolationKind::ContractMissing(contract.identifier.clone()),
                    Location::File(ctx.tree.to_string_lossy().into_owned()),
                );
                violations.push(violation);
            };

            // The matching solidity contract is missing, so we're done in
            // any case.
            return violations;
        }

        // We know a contract exists in both trees.
        let contract_hir = contract_hir.unwrap();
        let contract_sol = contract_sol.unwrap();
        if let Hir::ContractDefinition(contract_hir) = contract_hir {
            violations.append(&mut check_contract_names(
                contract_hir,
                &contract_sol,
                ctx,
            ));
            violations.append(&mut check_fns_structure(
                contract_hir,
                &contract_sol,
                ctx,
            ));
        };

        violations
    }
}

/// Checks that contract names match.
fn check_contract_names(
    contract_hir: &hir::ContractDefinition,
    contract_sol: &pt::ContractDefinition,
    ctx: &Context,
) -> Vec<Violation> {
    let mut violations = Vec::with_capacity(1);

    // We won't deal right now with a parsing error from `solang_parser`.
    if let Some(ref identifier) = contract_sol.name {
        let contract_name = sanitize(&contract_hir.identifier);
        if identifier.name != contract_name {
            let violation = Violation::new(
                ViolationKind::ContractNameNotMatches(
                    contract_name,
                    identifier.name.clone(),
                ),
                Location::Code(
                    ctx.sol.as_path().to_string_lossy().into_owned(),
                    offset_to_line(&ctx.src, contract_sol.loc.start()),
                ),
            );
            violations.push(violation);
        }
    };

    violations
}

/// Checks that function structures match between the HIR and the Solidity AST.
/// i.e. that all the functions are present in the output file in the right
/// order. This could be better, currently it is O(N^2).
fn check_fns_structure(
    contract_hir: &hir::ContractDefinition,
    contract_sol: &pt::ContractDefinition,
    ctx: &Context,
) -> Vec<Violation> {
    let mut violations = Vec::new();

    // Check that hir functions are present in the solidity contract. Store
    // their indices for later processing.
    let mut present_fn_indices =
        Vec::with_capacity(contract_hir.children.len());
    for (hir_idx, fn_hir) in contract_hir.children.iter().enumerate() {
        if let Hir::FunctionDefinition(fn_hir) = fn_hir {
            let fn_sol = find_matching_fn(contract_sol, fn_hir);

            match fn_sol {
                // Store the matched function to check it is at the
                // appropriate place later.
                Some((sol_idx, _)) => {
                    present_fn_indices.push((hir_idx, sol_idx))
                }
                // We didn't find a matching function, so this is a
                // violation.
                None => {
                    if !ctx.cfg.skip_modifiers {
                        violations.push(Violation::new(
                            ViolationKind::MatchingFunctionMissing(
                                fn_hir.clone(),
                                hir_idx,
                            ),
                            Location::Code(
                                ctx.tree.to_string_lossy().into_owned(),
                                fn_hir.span.start.line,
                            ),
                        ))
                    }
                }
            }
        };
    }

    // No matching constructs were found. We can just return, since
    // we already processed violations in the prev step.
    if present_fn_indices.is_empty() {
        return violations;
    }

    let mut unsorted_set: BTreeSet<(usize, usize)> = BTreeSet::new();
    // We need to check for inversions in order to know if the order is wrong.
    for i in 0..present_fn_indices.len() - 1 {
        let (i_hir_idx, i_sol_idx) = present_fn_indices[i];
        // Everything that's less than the ith item is unsorted.
        // If there is at least one element that is less than the
        // ith item, then, this element is also unsorted.
        for j in i + 1..present_fn_indices.len() {
            let (_, j_sol_idx) = present_fn_indices[j];
            // We found an inversion.
            if i_sol_idx > j_sol_idx {
                unsorted_set.insert((i_hir_idx, i_sol_idx));
            }
        }
    }

    // Emit a violation per unsorted item.
    for (hir_idx, sol_idx) in unsorted_set {
        if let Hir::FunctionDefinition(_) = contract_hir.children[hir_idx] {
            if let pt::ContractPart::FunctionDefinition(ref fn_sol) =
                contract_sol.parts[sol_idx]
            {
                violations.push(Violation::new(
                    ViolationKind::FunctionOrderMismatch(
                        *fn_sol.clone(),
                        sol_idx,
                        hir_idx,
                    ),
                    Location::Code(
                        ctx.sol.clone().to_string_lossy().into_owned(),
                        offset_to_line(&ctx.src, fn_sol.loc.start()),
                    ),
                ));
            }
        }
    }

    violations
}
