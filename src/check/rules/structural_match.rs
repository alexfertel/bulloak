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
pub(crate) struct StructuralMatcher;

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

        // Check that contract names match.
        if let Hir::ContractDefinition(contract_hir) = contract_hir {
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

                // Check that all the functions are present in the
                // output file with the right order.
                violations.append(&mut check_fns_structure(contract_hir, contract_sol)?);
            };
        };

        Ok(violations)
    }
}

/// Checks that function structures match between the HIR and the solidity AST.
///
/// This could be better, currently it is O(N^2).
fn check_fns_structure(
    contract_hir: &hir::ContractDefinition,
    contract_sol: &pt::ContractDefinition,
) -> anyhow::Result<Vec<Violation>> {
    let mut violations = Vec::new();

    let mut present_fns = Vec::with_capacity(contract_hir.children.len());
    for fn_hir in &contract_hir.children {
        if let Hir::FunctionDefinition(fn_hir) = fn_hir {
            let fn_sol = find_matching_fn(contract_sol, fn_hir);

            match fn_sol {
                // Store the matched function to check its at the
                // appropriate place later.
                Some(fn_sol) => present_fns.push(fn_sol),
                // We didn't find a matching function, so this is a
                // violation.
                None => violations.push(Violation::new(ViolationKind::MatchingTestMissing(
                    fn_hir.identifier.clone(),
                ))),
            }
        };
    }

    for (fn_hir, fn_sol) in contract_hir
        .children
        .iter()
        .filter_map(|fn_hir| {
            if let Hir::FunctionDefinition(fn_hir) = fn_hir {
                Some(fn_hir)
            } else {
                None
            }
        })
        .zip(present_fns.iter())
    {
        if !fns_match(fn_hir, fn_sol) {
            violations.push(Violation::new(ViolationKind::TestOrderMismatch(
                fn_hir.identifier.clone(),
            )));
        }
    }

    Ok(violations)
}

/// Performs a search over the sol contract parts trying to find
/// the matching bulloak function.
///
/// Two functions match if they have the same name and their types match.
fn find_matching_fn<'a>(
    contract_sol: &'a pt::ContractDefinition,
    fn_hir: &'a hir::FunctionDefinition,
) -> Option<&'a Box<pt::FunctionDefinition>> {
    contract_sol.parts.iter().find_map(|part| {
        if let pt::ContractPart::FunctionDefinition(fn_sol) = part {
            if fns_match(fn_hir, fn_sol) {
                return Some(fn_sol);
            }
        };

        None
    })
}

/// Check whether a solidity function matches its bulloak counterpart.
///
/// Two functions match if they have the same name and their types match.
fn fns_match(fn_hir: &hir::FunctionDefinition, fn_sol: &pt::FunctionDefinition) -> bool {
    fn_sol
        .name
        .clone()
        .is_some_and(|pt::Identifier { ref name, .. }| {
            name == &fn_hir.identifier && fn_types_match(&fn_hir.ty, &fn_sol.ty)
        })
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use solang_parser::pt;

    use crate::check::rules::structural_match::{find_matching_fn, fn_types_match, fns_match};
    use crate::hir;

    #[test]
    fn test_fn_types_match() {
        assert!(fn_types_match(
            &hir::FunctionTy::Function,
            &pt::FunctionTy::Function
        ));
        assert!(fn_types_match(
            &hir::FunctionTy::Modifier,
            &pt::FunctionTy::Modifier
        ));
    }

    fn fn_hir(name: &str, ty: hir::FunctionTy) -> hir::FunctionDefinition {
        hir::FunctionDefinition {
            identifier: name.to_owned(),
            ty,
            modifiers: Default::default(),
            children: Default::default(),
        }
    }

    fn fn_sol(name: &str, ty: pt::FunctionTy) -> pt::FunctionDefinition {
        pt::FunctionDefinition {
            name: Some(pt::Identifier::new(name)),
            ty,
            loc: Default::default(),
            name_loc: Default::default(),
            params: Default::default(),
            attributes: Default::default(),
            return_not_returns: Default::default(),
            returns: Default::default(),
            body: Default::default(),
        }
    }

    #[test]
    fn test_fns_match() {
        assert!(fns_match(
            &fn_hir("my_fn", hir::FunctionTy::Function),
            &fn_sol("my_fn", pt::FunctionTy::Function)
        ));
        assert!(!fns_match(
            &fn_hir("my_fn", hir::FunctionTy::Function),
            &fn_sol("not_my_fn", pt::FunctionTy::Function)
        ));
        assert!(!fns_match(
            &fn_hir("not_my_fn", hir::FunctionTy::Function),
            &fn_sol("my_fn", pt::FunctionTy::Function)
        ));
        assert!(fns_match(
            &fn_hir("my_fn", hir::FunctionTy::Modifier),
            &fn_sol("my_fn", pt::FunctionTy::Modifier)
        ));
        assert!(!fns_match(
            &fn_hir("my_fn", hir::FunctionTy::Modifier),
            &fn_sol("my_fn", pt::FunctionTy::Function)
        ));
        assert!(!fns_match(
            &fn_hir("my_fn", hir::FunctionTy::Function),
            &fn_sol("my_fn", pt::FunctionTy::Modifier)
        ));
    }

    fn fn_sol_as_part(name: &str, ty: pt::FunctionTy) -> pt::ContractPart {
        pt::ContractPart::FunctionDefinition(Box::new(fn_sol(name, ty)))
    }

    #[test]
    fn test_find_matching_fn() {
        let needle_sol = fn_sol("needle", pt::FunctionTy::Function);
        let haystack = vec![
            fn_sol_as_part("hay", pt::FunctionTy::Function),
            fn_sol_as_part("more_hay", pt::FunctionTy::Function),
            fn_sol_as_part("needle", pt::FunctionTy::Function),
            fn_sol_as_part("hay_more", pt::FunctionTy::Function),
        ];
        let needle_hir = fn_hir("needle", hir::FunctionTy::Function);
        let contract = pt::ContractDefinition {
            loc: Default::default(),
            ty: pt::ContractTy::Contract(Default::default()),
            name: Default::default(),
            base: Default::default(),
            parts: haystack,
        };

        let expected = needle_sol;
        let actual = find_matching_fn(&contract, &needle_hir).unwrap();
        assert_eq!(&Box::new(expected), actual);

        let haystack = vec![];
        let needle_hir = fn_hir("needle", hir::FunctionTy::Function);
        let contract = pt::ContractDefinition {
            loc: Default::default(),
            ty: pt::ContractTy::Contract(Default::default()),
            name: Default::default(),
            base: Default::default(),
            parts: haystack,
        };

        let actual = find_matching_fn(&contract, &needle_hir);
        assert_eq!(None, actual);
    }
}
