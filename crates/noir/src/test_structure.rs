//! Noir testfile representation
use anyhow::{bail, Result};
use bulloak_syntax::{Action, Ast};
use std::collections::HashSet;

use crate::{
    constants::{PANIC_KEYWORDS, TEST_PREFIX},
    utils::{parse_root_name, to_snake_case},
};

#[derive(Debug)]
pub(crate) struct Root {
    pub modules: Vec<Module>,
    pub functions: Vec<Function>,
}

impl Root {
    pub(crate) fn new(forest: &Vec<Ast>) -> Result<Root> {
        let mut modules = Vec::new();
        let mut functions = Vec::<Function>::new();
        let mut all_hooks: HashSet<String> = HashSet::new();
        let mut repeated_hooks: HashSet<String> = HashSet::new();
        match forest.iter().len() {
            0 => Ok(Root { functions, modules }),
            1 => {
                let test_functions = collect_tests(forest, &[]);

                functions.extend(
                    collect_helpers(&test_functions)
                        .into_iter()
                        .map(|x| Function::SetupHook(x))
                        .collect::<Vec<_>>(),
                );
                functions.extend(
                    test_functions
                        .into_iter()
                        .map(|x| Function::TestFunction(x))
                        .collect::<Vec<_>>(),
                );
                Ok(Root { functions, modules })
            }
            _ => {
                let mut names: HashSet<String> = HashSet::new();
                for (index, ast) in forest.iter().enumerate() {
                    let Ast::Root(root) = ast else {
                        panic!("AST forest should start with roots")
                    };
                    let (module_name, name) =
                        parse_root_name(&root.contract_name);
                    let Some(name) = name else {
                        bail!(
                            r#"an error occurred while parsing the tree: separator missing at tree root #{} "{}". Expected to find `::` between the contract name and the function name when multiple roots exist"#,
                            index + 1, // solidity backend uses 1-indexing
                            module_name
                        );
                    };
                    if !names.insert(name.clone()) {
                        bail!(
                            "submodule {} has more than one definition",
                            name
                        );
                    }

                    let local_tests =
                        collect_tests(std::slice::from_ref(ast), &[]);

                    let helpers = collect_helpers(&local_tests);
                    for hook in &helpers {
                        // returns false if the key is already present
                        if !all_hooks.insert(hook.name.clone()) {
                            // we don't care if it's repeated one or multiple times
                            repeated_hooks.insert(hook.name.clone());
                        }
                    }

                    let mut local_functions = Vec::new();
                    local_functions.extend(helpers
                            .into_iter()
                            .map(|x| Function::SetupHook(x))
                            .collect::<Vec<_>>(),
                    );
                    local_functions.extend(
                        local_tests
                            .into_iter()
                            .map(|x| Function::TestFunction(x))
                            .collect::<Vec<_>>(),
                    );
                    modules.push(Module { name, functions: local_functions });
                }

                modules = modules
                    .into_iter()
                    .map(|module| Module {
                        name: module.name.clone(),
                        functions: module
                            .functions
                            .into_iter()
                            .filter(|fun| !repeated_hooks.contains(&fun.name()))
                            .collect(),
                    })
                    .collect();

                Ok(Root {
                    modules,
                    functions: repeated_hooks
                        .into_iter()
                        .map(|name| Function::SetupHook(SetupHook { name }))
                        .collect(),
                })
            }
        }
    }
}

fn collect_helpers(test_functions: &Vec<TestFunction>) -> Vec<SetupHook> {
    let mut hooks = Vec::new();
    let mut all_hooks: HashSet<String> = HashSet::new();
    for func in test_functions {
        for hook in &func.setup_hooks {
            // skip repeated hooks
            if all_hooks.insert(hook.name.clone()) {
                hooks.push(hook.clone());
            }
        }
    }
    hooks
}

/// Used for both definition and invocation
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Debug)]
pub(crate) struct SetupHook {
    pub name: String,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Debug)]
pub(crate) struct Module {
    pub name: String,
    pub functions: Vec<Function>,
}

#[derive(Debug, Hash, PartialEq, PartialOrd, Ord, Eq, Clone)]
pub(crate) struct TestFunction {
    pub name: String,
    pub expect_fail: bool,
    pub setup_hooks: Vec<SetupHook>,
    pub actions: Vec<String>,
}

#[derive(PartialEq, Eq, Hash, Clone, PartialOrd, Ord, Debug)]
pub(crate) enum Function {
    SetupHook(SetupHook),
    TestFunction(TestFunction),
}

impl Function {
    pub(crate) fn name(&self) -> String {
        match self {
            Function::TestFunction(f) => f.name.clone(),
            Function::SetupHook(h) => h.name.clone(),
        }
    }
}

fn collect_tests(
    children: &[Ast],
    parent_helpers: &[SetupHook],
) -> Vec<TestFunction> {
    let mut tests = Vec::new();

    for child in children {
        match child {
            Ast::Condition(condition) => {
                let mut helpers = parent_helpers.to_vec();
                helpers
                    .push(SetupHook { name: to_snake_case(&condition.title) });
                // Cllect all direct Action children
                let actions: Vec<&Action> = condition
                    .children
                    .iter()
                    .filter_map(|c| match c {
                        Ast::Action(a) => Some(a),
                        _ => None,
                    })
                    .collect();

                // Generate ONE test function for all actions under this
                // condition
                if !actions.is_empty() {
                    // use the last helper only if the action has silbings
                    // that merit re-using it
                    tests.push(generate_test_function(
                        &actions,
                        &helpers,
                        condition.children.len() > 1,
                    ));
                }

                // Process only nested Condition children (not actions!)
                // recursively We need to collect into a Vec
                // first, then pass a slice
                let nested_conditions: Vec<_> = condition
                    .children
                    .iter()
                    .filter(|c| matches!(c, Ast::Condition(_)))
                    .collect();

                for nested_cond in nested_conditions {
                    tests.extend(collect_tests(
                        std::slice::from_ref(nested_cond),
                        &helpers,
                    ));
                }
            }
            Ast::Action(action) => {
                // Root-level action
                tests.push(generate_test_function(
                    &[action],
                    &parent_helpers.to_vec(),
                    false,
                ));
            }
            Ast::Root(root) => {
                tests.extend(collect_tests(&root.children, &[]));
            }
            _ => {}
        }
    }

    tests
}

/// Generate a single test function for one or more actions.
fn generate_test_function(
    actions: &[&Action],
    helpers: &Vec<SetupHook>,
    use_last_helper: bool,
) -> TestFunction {
    // Determine test name
    let name = if helpers.is_empty() {
        let title = &actions[0].title;
        // trim 'it' from first-level assertions (not very frequent, but necessary for consistency
        // with foundry backend)
        let title = title
            .strip_prefix("it ")
            .or_else(|| title.strip_prefix("It "))
            .unwrap_or(title);
        // Root level: test_{action_name}
        format!("{}_{}", TEST_PREFIX, to_snake_case(title))
    } else {
        // Under condition: test_{last_helper}
        format!("{}_{}", TEST_PREFIX, &helpers.last().unwrap().name)
    };

    let expect_fail =
        actions.iter().any(|action| has_panic_keyword(&action.title));

    let setup_hooks: Vec<SetupHook> = helpers
        .iter()
        .filter(|x| *x != helpers.iter().last().unwrap() || use_last_helper)
        .cloned()
        .collect();

    TestFunction {
        name,
        expect_fail,
        setup_hooks,
        // TODO: c'mon the action array is right there...
        actions: actions.iter().map(|x| x.title.clone()).collect(),
    }
}

/// Check if a title contains panic keywords.
fn has_panic_keyword(title: &str) -> bool {
    let lower = title.to_lowercase();
    PANIC_KEYWORDS.iter().any(|keyword| lower.contains(keyword))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bulloak_syntax::parse;

    #[test]
    fn test_root_new_empty() {
        let forest = vec![];
        let root = Root::new(&forest).unwrap();
        assert!(root.modules.is_empty());
        assert!(root.functions.is_empty());
    }

    #[test]
    fn test_root_new_single_tree() {
        let tree = r"
test_root
└── It should work.
";
        let forest = parse(tree).unwrap();
        let root = Root::new(&forest).unwrap();

        assert!(root.modules.is_empty());
        assert_eq!(root.functions.len(), 1);
        match &root.functions[0] {
            Function::TestFunction(f) => {
                assert_eq!(f.name, "test_should_work");
                assert!(!f.expect_fail);
                assert!(f.setup_hooks.is_empty());
            }
            _ => panic!("Expected TestFunction"),
        }
    }

    #[test]
    fn test_root_with_special_characters_in_submodule() {
        let tree = r"
TestRoot::foo==bar
└── It should work fine

TestRoot::foo==baz
└── It should also work
";
        let forest = parse(tree).unwrap();
        let root = Root::new(&forest).unwrap();

        assert_eq!(root.modules.len(), 2);
        assert!(root.functions.is_empty());

        assert_eq!(root.modules[0].name, "foobar");
        assert_eq!(root.modules[1].name, "foobaz");

        for module in &root.modules {
            assert_eq!(module.functions.len(), 1);
        }
    }

    #[test]
    fn test_collect_helpers() {
        let tree = r"
test_root
└── When A
    └── When B
        ├── It should work.
        └── When C
            └── It should also work.
";
        let forest = parse(tree).unwrap();
        let root = Root::new(&forest).unwrap();

        assert_eq!(root.functions.len(), 4);
        assert_eq!(root.functions[0].name(), "when_a");
        assert!(matches!(root.functions[0], Function::SetupHook(_)));
        assert_eq!(root.functions[1].name(), "when_b");
        assert!(matches!(root.functions[1], Function::SetupHook(_)));
        assert_eq!(root.functions[2].name(), "test_when_b");
        assert!(matches!(root.functions[2], Function::TestFunction(_)));
        assert_eq!(root.functions[3].name(), "test_when_c");
        assert!(matches!(root.functions[3], Function::TestFunction(_)));
    }

    /// this borders on testing a bug, since this is always checked by the calling function, since
    /// it knows the filenames and can report if the root doesn't match
    #[test]
    fn test_multiple_roots_missing_separator() {
        let tree = r"
FirstRoot
└── It should work

FirstRoot
└── It should also work
";
        let forest = parse(tree).unwrap();
        let result = Root::new(&forest);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("separator missing"));
    }

    #[test]
    fn test_duplicate_submodule_names_reports_on_sanitized_name() {
        let tree = r"
Contract::Module1
└── It should work

Contract::Module1
└── It should also work
";
        let forest = parse(tree).unwrap();
        let result = Root::new(&forest);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err
            .to_string()
            .contains("submodule Module1 has more than one definition"));
    }

    #[test]
    fn test_duplicate_submodule_names_after_sanitization() {
        let tree = r"
Contract::foo>bar
└── It should work

Contract::foo<bar
└── It should also work
";
        let forest = parse(tree).unwrap();
        let result = Root::new(&forest);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err
            .to_string()
            .contains("submodule foobar has more than one definition"));
    }

    // A proper HIR-based implementation may also create a setup hook for when_c
    #[test]
    fn test_hoist_shared_setup() {
        let tree = r"
Contract::foo
└── When B
    ├── It should work.
    └── When C
        └── It should also work.

Contract::bar
└── When A
    └── When B
        ├── It should produce a special side effect
        └── When C
            └── It should also work.
";
        let forest = parse(tree).unwrap();
        let root = Root::new(&forest).unwrap();

        assert_eq!(root.functions.iter().len(), 1);
        assert_eq!(root.functions[0].name(), "when_b");
        assert!(matches!(root.functions[0], Function::SetupHook(_)));

        let foo_module = &root.modules[0];
        assert_eq!(foo_module.name, "foo");
        assert_eq!(foo_module.functions.len(), 2);
        assert_eq!(foo_module.functions[0].name(), "test_when_b");
        assert!(matches!(foo_module.functions[0], Function::TestFunction(_)));
        assert_eq!(foo_module.functions[1].name(), "test_when_c");
        assert!(matches!(foo_module.functions[1], Function::TestFunction(_)));

        let bar_module = &root.modules[1];
        assert_eq!(bar_module.name, "bar");
        assert_eq!(bar_module.functions.len(), 3);
        assert_eq!(bar_module.functions[0].name(), "when_a");
        assert!(matches!(bar_module.functions[0], Function::SetupHook(_)));
        assert_eq!(bar_module.functions[1].name(), "test_when_b");
        assert!(matches!(bar_module.functions[1], Function::TestFunction(_)));
        assert_eq!(bar_module.functions[2].name(), "test_when_c");
        assert!(matches!(bar_module.functions[2], Function::TestFunction(_)));
    }

    /// Regression test for https://github.com/defi-wonderland/bulloak/pull/9#issuecomment-3710452952
    /// When multiple roots share the same leaf condition (a condition with only action children),
    /// the shared setup hook should be hoisted to root.functions.
    #[test]
    fn test_hoist_shared_leaf_condition_setup() {
        let tree = r"
hoisted_hook_regression::constructor_with_minter
└── when passing valid parameters
    ├── it sets name
    └── it sets symbol

hoisted_hook_regression::constructor_with_initial_supply
└── when passing valid parameters
    ├── it sets name
    └── it sets symbol
";
        let forest = parse(tree).unwrap();
        let root = Root::new(&forest).unwrap();

        // The shared condition should be hoisted to root.functions as a SetupHook
        assert_eq!(
            root.functions.len(),
            1,
            "Expected 1 hoisted setup hook, found {}. Root functions: {:?}",
            root.functions.len(),
            root.functions
        );
        assert_eq!(root.functions[0].name(), "when_passing_valid_parameters");
        assert!(
            matches!(root.functions[0], Function::SetupHook(_)),
            "Expected SetupHook, got {:?}",
            root.functions[0]
        );

        // Verify modules are created correctly
        assert_eq!(root.modules.len(), 2);

        let minter_module = &root.modules[0];
        assert_eq!(minter_module.name, "constructor_with_minter");
        assert_eq!(minter_module.functions.len(), 1);
        assert_eq!(
            minter_module.functions[0].name(),
            "test_when_passing_valid_parameters"
        );
        assert!(matches!(
            minter_module.functions[0],
            Function::TestFunction(_)
        ));

        let supply_module = &root.modules[1];
        assert_eq!(supply_module.name, "constructor_with_initial_supply");
        assert_eq!(supply_module.functions.len(), 1);
        assert_eq!(
            supply_module.functions[0].name(),
            "test_when_passing_valid_parameters"
        );
        assert!(matches!(
            supply_module.functions[0],
            Function::TestFunction(_)
        ));
    }

    /// Related to https://github.com/defi-wonderland/bulloak/pull/9#issuecomment-3710452952
    /// check the modifier would be generated in the case of a single-root testfile
    #[test]
    fn test_single_root_setup_hook_generation() {
        let tree = r"
hoisted_hook_regression::constructor_with_minter
└── when passing valid parameters
    ├── it sets name
    └── it sets symbol
";
        let forest = parse(tree).unwrap();
        let root = Root::new(&forest).unwrap();
        dbg!(&root);

        assert_eq!(
            root.functions.len(),
            2,
            "Expected 1 setup hook and 1 test function, found {}. Root functions: {:?}",
            root.functions.len(),
            root.functions
        );
        assert_eq!(root.functions[0].name(), "when_passing_valid_parameters");
        assert!(
            matches!(root.functions[0], Function::SetupHook(_)),
            "Expected SetupHook, got {:?}",
            root.functions[0]
        );

        // Verify no modules are defined
        assert_eq!(root.modules.len(), 0);

        assert_eq!(
            root.functions[1].name(),
            "test_when_passing_valid_parameters"
        );
        assert!(matches!(root.functions[1], Function::TestFunction(_)));
    }
}
