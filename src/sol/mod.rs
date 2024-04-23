//! This module implements functionality related to operating on a parse tree (PT) from `solang_parser`.

use solang_parser::pt::{
    ContractDefinition, ContractPart, FunctionDefinition, FunctionTy, Identifier, SourceUnit,
    SourceUnitPart,
};

use crate::hir::hir;
pub(crate) mod fmt;
pub(crate) mod translator;
mod visitor;

pub(crate) use fmt::Formatter;
pub(crate) use translator::Translator;

/// Searches for and returns the first `ContractDefinition` found in a given `SourceUnit`.
pub(crate) fn find_contract(pt: &SourceUnit) -> Option<Box<ContractDefinition>> {
    pt.0.iter().find_map(|part| match part {
        SourceUnitPart::ContractDefinition(contract) => Some(contract.clone()),
        _ => None,
    })
}

/// Given a HIR function, `find_matching_fn` performs a search over the sol
/// contract parts trying to find a sol function with a matching name and type.
pub(crate) fn find_matching_fn<'a>(
    contract_sol: &'a ContractDefinition,
    fn_hir: &'a hir::FunctionDefinition,
) -> Option<(usize, &'a FunctionDefinition)> {
    contract_sol
        .parts
        .iter()
        .enumerate()
        .find_map(|(idx, part)| {
            if let ContractPart::FunctionDefinition(fn_sol) = part {
                if fns_match(fn_hir, fn_sol) {
                    return Some((idx, &**fn_sol));
                }
            };

            None
        })
}

/// Check whether a Solidity function matches its bulloak counterpart.
///
/// Two functions match if they have the same name and their types match.
fn fns_match(fn_hir: &hir::FunctionDefinition, fn_sol: &FunctionDefinition) -> bool {
    fn_sol
        .name
        .clone()
        .is_some_and(|Identifier { ref name, .. }| {
            name == &fn_hir.identifier && fn_types_match(&fn_hir.ty, fn_sol.ty)
        })
}

/// Checks that the function types between a HIR function
/// and a `solang_parser` function match.
///
/// We check that the function types match, even though we know that the
/// name not matching is enough, since a modifier will never be
/// named the same as a function per Foundry's best practices.
const fn fn_types_match(ty_hir: &hir::FunctionTy, ty_sol: FunctionTy) -> bool {
    match ty_hir {
        hir::FunctionTy::Function => matches!(ty_sol, FunctionTy::Function),
        hir::FunctionTy::Modifier => matches!(ty_sol, FunctionTy::Modifier),
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use solang_parser::pt;

    use crate::{
        hir,
        sol::{find_matching_fn, fn_types_match, fns_match},
    };

    #[test]
    fn test_fn_types_match() {
        assert!(fn_types_match(
            &hir::FunctionTy::Function,
            pt::FunctionTy::Function
        ));
        assert!(fn_types_match(
            &hir::FunctionTy::Modifier,
            pt::FunctionTy::Modifier
        ));
    }

    fn fn_hir(name: &str, ty: hir::FunctionTy) -> hir::FunctionDefinition {
        hir::FunctionDefinition {
            identifier: name.to_owned(),
            ty,
            span: Default::default(),
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
        assert_eq!((2, &expected), actual);

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
