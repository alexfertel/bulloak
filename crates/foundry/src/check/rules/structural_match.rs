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
use solang_parser::pt::{self, ContractPart};

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
/// Read more at the [module-level documentation][self].
pub struct StructuralMatcher;

impl Checker for StructuralMatcher {
    fn check(ctx: &Context) -> Vec<Violation> {
        let mut violations = vec![];

        // We support multiple trees per .tree file, but they are combined into
        // a single HIR during the [`hir::translate`]  step when creating the
        // context which means that there can only be one contract. This is
        // reflected in the current tree hierarchy of the HIR.
        let Some(contract_hir) = ctx.hir.find_contract() else {
            // If there is no contract in the .tree file, then we don't check
            // anything.
            return violations;
        };

        // Find the first occurrence of a contract.
        let Some(contract_sol) = find_contract(&ctx.pt) else {
            // If we find no contract in the Solidity file, then there must
            // be no contract in the HIR, else we found a violation.
            let violation = Violation::new(
                ViolationKind::ContractMissing(contract_hir.identifier.clone()),
                Location::File(ctx.tree.to_string_lossy().into_owned()),
            );
            violations.push(violation);

            // The matching solidity contract is missing, so we're done.
            return violations;
        };

        // We know a contract exists in both trees.
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
        let Hir::Function(fn_hir) = fn_hir else {
            continue;
        };

        let maybe_matching_fn = find_matching_fn(contract_sol, fn_hir);
        let Some((sol_idx, _)) = maybe_matching_fn else {
            // We didn't find a matching function, so this is a
            // violation.

            // If the missing function is a modifier we don't actually want
            // to emit it if the `skip_modifiers` flag is set.
            if ctx.cfg.skip_modifiers && fn_hir.is_modifier() {
                continue;
            }

            violations.push(Violation::new(
                ViolationKind::MatchingFunctionMissing(fn_hir.clone(), hir_idx),
                Location::Code(
                    ctx.tree.to_string_lossy().into_owned(),
                    fn_hir.span.start.line,
                ),
            ));

            continue;
        };

        // Store the matched function to check it is at the
        // appropriate place later.
        present_fn_indices.push((hir_idx, sol_idx));
    }

    // No matching constructs were found. We can just return since we already
    // processed violations in the prev step.
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
        let Hir::Function(_) = contract_hir.children[hir_idx] else {
            continue;
        };

        let fn_sol = &contract_sol.parts[sol_idx];
        let ContractPart::FunctionDefinition(fn_sol) = fn_sol else {
            continue;
        };

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

    violations
}

#[cfg(test)]
mod tests {
    use super::StructuralMatcher;
    use crate::check::rules::Checker;
    use crate::{
        check::context::Context, check::violation::ViolationKind,
        config::Config,
    };
    use std::{fs, io::Write, path::PathBuf};
    use tempfile::tempdir;

    /// Helper: write a file under `dir` and return its PathBuf.
    fn write_file(
        dir: &std::path::Path,
        name: &str,
        contents: &str,
    ) -> PathBuf {
        let path = dir.join(name);
        let mut f = fs::File::create(&path).unwrap();
        write!(f, "{}", contents).unwrap();
        path
    }

    /// Build a Context from given .tree and .t.sol text.
    fn make_ctx(tree_src: &str, sol_src: &str) -> Context {
        let td = tempdir().unwrap();
        let tree = write_file(td.path(), "X.tree", tree_src);
        let sol = td.path().join("X.t.sol");
        fs::write(&sol, sol_src).unwrap();
        let mut cfg = Config::default();
        cfg.files = vec![tree.clone()];
        Context::new(tree, &cfg).unwrap()
    }

    #[test]
    fn contract_name_mismatch_detected() {
        let tree = r#"
Foo
└── It should do something.
"#;
        let sol = r#"
// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;
contract Bar {
  function test_ShouldDoSomething() external {
    // It should do something.
  }
}
"#;
        let ctx = make_ctx(tree, sol);
        let v = StructuralMatcher::check(&ctx);
        assert_eq!(1, v.len(), "expected exactly one violation");
        match &v[0].kind {
            ViolationKind::ContractNameNotMatches(expected, found) => {
                assert_eq!(expected, "Foo");
                assert_eq!(found, "Bar");
            }
            other => panic!("wrong violation kind: {:?}", other),
        }
    }

    #[test]
    fn missing_test_function() {
        let tree = r#"
Foo
├── It one.
└── It two.
"#;
        let sol = r#"
// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;
contract Foo {
  function test_One() external {
    // It one.
  }
}
"#;
        let ctx = make_ctx(tree, sol);
        let mut v = StructuralMatcher::check(&ctx);
        // We only get one Missing for "test_Two".
        assert_eq!(1, v.len());
        match &v.pop().unwrap().kind {
            ViolationKind::MatchingFunctionMissing(fn_hir, _) => {
                assert!(
                    fn_hir.identifier.contains("Two"),
                    "expected missing `Two` function"
                );
            }
            other => {
                panic!("expected MatchingFunctionMissing, got {:?}", other)
            }
        }
    }

    #[test]
    fn function_order_violation() {
        let tree = r#"
Foo
├── It one.
└── It two.
"#;
        let sol = r#"
// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;
contract Foo {
  function test_Two() external {
    // It two.
  }
  function test_One() external {
    // It one.
  }
}
"#;
        let ctx = make_ctx(tree, sol);
        // first pass will detect both presence and order issues.
        let vs = StructuralMatcher::check(&ctx);
        // Should at least include an order mismatch for test_One
        assert!(
            vs.iter().any(|v| matches!(
                v.kind,
                ViolationKind::FunctionOrderMismatch(_, _, _)
            )),
            "expected a FunctionOrderMismatch"
        );
    }

    #[test]
    fn skip_modifiers_flag_ignores_missing_modifiers() {
        let tree = r#"
Foo
└── When cond
    └── It does stuff.
"#;
        // This .t.sol has no modifier for `whenCond`
        let sol = r#"
// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;
contract Foo {
  function test_WhenCond() external {
    // It does stuff.
  }
}
"#;
        let td = tempdir().unwrap();
        let tree = write_file(td.path(), "A.tree", tree);
        let sol_path = td.path().join("A.t.sol");
        fs::write(&sol_path, sol).unwrap();

        let mut cfg = Config::default();
        cfg.skip_modifiers = true;
        cfg.files = vec![tree.clone()];

        let ctx = Context::new(tree.clone(), &cfg).unwrap();
        let vs = StructuralMatcher::check(&ctx);
        // Since we skipped modifiers, no violations at all.
        assert!(
            vs.is_empty(),
            "expected no violations when skip_modifiers=true"
        );
    }

    #[test]
    fn green_path() {
        let tree = r#"Foo
├── When cond
│  └── It does.
└── It final.
"#;
        let sol = r#"// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;
contract Foo {
  modifier whenCond() {
    _;
  }

  function test_WhenCond() external whenCond {
    // It does.
  }

  function test_Final() external {
    // It final.
  }
}
"#;
        let ctx = make_ctx(tree, sol);
        assert!(StructuralMatcher::check(&ctx).is_empty());
    }

    #[test]
    fn missing_modifier() {
        let tree = r#"Foo
└── When a
    └── When m
        └── It x.
"#;
        let sol = r#"// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;

contract Foo {
    function test_WhenM() {
        // It x.
    }
}
"#;
        let ctx = make_ctx(tree, sol);
        let vs = StructuralMatcher::check(&ctx);
        assert_eq!(1, vs.len());

        if let ViolationKind::MatchingFunctionMissing(fh, _) = &vs[0].kind {
            assert!(fh.is_modifier(), "{fh:?}");
        } else {
            panic!("expected missing modifier");
        }
    }

    #[test]
    fn extra_user_items_no_violation() {
        let tree = r#"Foo
└── It one.
"#;
        let sol = r#"// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;
contract Foo {
  function test_One() external {
    // It one.
  }

  modifier extraMod() {
    _;
  }

  function extraFn() external {}
}
"#;
        let ctx = make_ctx(tree, sol);
        assert!(StructuralMatcher::check(&ctx).is_empty());
    }

    #[test]
    fn missing_contract_entirely() {
        let tree = r#"Foo
└── It one.
"#;
        let sol = r#"// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;
// no contract here
"#;
        let ctx = make_ctx(tree, sol);
        let vs = StructuralMatcher::check(&ctx);
        assert_eq!(1, vs.len());
        if let ViolationKind::ContractMissing(name) = &vs[0].kind {
            assert_eq!(name, "Foo");
        } else {
            panic!("expected ContractMissing");
        }
    }

    #[test]
    fn three_way_order_mismatch() {
        let tree = r#"Foo
├── It A.
├── It B.
└── It C.
"#;
        let sol = r#"// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;
contract Foo {
  function test_A() external {
    // It A.
  }
  function test_C() external {
    // It C.
  }
  function test_B() external {
    // It B.
  }
}
"#;
        let ctx = make_ctx(tree, sol);
        let vs = StructuralMatcher::check(&ctx);
        let count = vs
            .iter()
            .filter(|v| {
                matches!(v.kind, ViolationKind::FunctionOrderMismatch(_, _, _))
            })
            .count();
        assert_eq!(1, count);
    }

    #[test]
    fn mixed_modifiers_and_functions_order() {
        let tree = r#"Foo
├── When X
│  └── When three
│      └── It one.
└── When Y
    └── When four
        └── It two.
"#;
        let sol = r#"// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;
contract Foo {
  modifier whenY() {
    _;
  }
  modifier whenX() {
    _;
  }
  function test_WhenFour() external whenY {
    // It one.
  }
  function test_WhenThree() external whenX {
    // It two.
  }
}
"#;
        let ctx = make_ctx(tree, sol);
        let vs = StructuralMatcher::check(&ctx);
        let count = vs
            .iter()
            .filter(|v| {
                matches!(v.kind, ViolationKind::FunctionOrderMismatch(_, _, _))
            })
            .count();
        assert_eq!(2, count);
    }

    #[test]
    fn sanitization_roundtrip() {
        let tree = r#"Foo
└── When stuff-happens
    └── It ok.
"#;
        let sol = r#"// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;
contract Foo {
  modifier whenStuff_happens() {
    _;
  }
  function test_WhenStuff_happens() external whenStuff_happens {
    // It ok.
  }
}
"#;
        let ctx = make_ctx(tree, sol);
        assert!(StructuralMatcher::check(&ctx).is_empty());
    }

    #[test]
    fn skip_modifiers_toggle() {
        // no conditions
        let tree = r#"Foo
├── It one.
└── It two.
"#;
        let sol = r#"// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;
contract Foo {
  function test_One() external { /*...*/ }
  function test_Two() external { /*...*/ }
}
"#;
        let mut cfg = Config::default();
        cfg.skip_modifiers = true;
        cfg.files =
            vec![write_file(&tempdir().unwrap().path(), "Z.tree", tree)];
        let td = tempdir().unwrap();
        let treep = write_file(td.path(), "Z.tree", tree);
        let solp = td.path().join("Z.t.sol");
        fs::write(&solp, sol).unwrap();
        let ctx1 = Context::new(treep.clone(), &cfg).unwrap();
        assert!(StructuralMatcher::check(&ctx1).is_empty());

        cfg.skip_modifiers = false;
        cfg.files = vec![treep.clone()];
        let ctx2 = Context::new(treep, &cfg).unwrap();
        assert!(StructuralMatcher::check(&ctx2).is_empty());
    }

    #[test]
    fn multiple_roots_combined() {
        let tree = r#"Foo::a
└── It A.

Foo::b
└── It B.
"#;
        let sol = r#"// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;
contract Foo {
  function test_AA() external {
    // It A.
  }
  function test_BB() external {
    // It B.
  }
}
"#;
        let ctx = make_ctx(tree, sol);
        assert_eq!(StructuralMatcher::check(&ctx), vec![]);
    }

    #[test]
    fn roundtrip_fix_no_violations() {
        let tree = r#"Foo
└── It one.
"#;
        let sol = r#"// SPDX-License-Identifier: UNLICENSE-Identifier
pragma solidity 0.8.0;
contract Bar {
  function test_One() external {
    // It one.
  }
}
"#;
        let ctx0 = make_ctx(tree, sol);
        let vs = StructuralMatcher::check(&ctx0);
        assert!(!vs.is_empty());
        let ctx1 = vs.into_iter().fold(ctx0, |ctx, v| v.kind.fix(ctx).unwrap());
        assert!(StructuralMatcher::check(&ctx1).is_empty());
    }
}
