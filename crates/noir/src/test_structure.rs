//! Noir testfile representation
use bulloak_syntax::{Action, Ast};
use std::collections::HashSet;

use crate::{
    constants::{PANIC_KEYWORDS, TEST_PREFIX},
    utils::to_snake_case,
};

pub(crate) struct Root {
    pub modules: Vec<Module>,
    pub functions: Vec<Function>,
}

impl Root {
    pub(crate) fn new(forest: &Vec<Ast>) -> Root {
        let mut modules = Vec::new();
        let mut functions = Vec::<Function>::new();
        match forest.iter().len() {
            0 => Root { functions, modules },
            1 => {
                functions.extend(
                    collect_helpers(forest)
                        .into_iter()
                        .map(|x| Function::SetupHook(x)),
                );
                functions.extend(
                    collect_tests(forest, &[])
                        .into_iter()
                        .map(|x| Function::TestFunction(x)),
                );
                Root { functions, modules }
            }
            _ => {
                for ast in forest {
                    let Ast::Root(root) = ast else {
                        panic!("AST forest should start with roots")
                    };
                    let submodule_name = get_submodule_name(
                        &root.contract_name,
                    )
                    .unwrap_or_else(|| todo!("when better parsing names"));

                    let tree_slice = std::slice::from_ref(ast);
                    let helpers = collect_helpers(tree_slice);

                    let mut local_functions = Vec::new();
                    local_functions.extend(
                        helpers.into_iter().map(|x| Function::SetupHook(x)),
                    );
                    local_functions.extend(
                        collect_tests(tree_slice, &[])
                            .into_iter()
                            .map(|x| Function::TestFunction(x)),
                    );

                    modules.push(Module {
                        name: submodule_name.to_string(),
                        functions: local_functions,
                    });
                }
                Root { modules, functions: Vec::new() }
            }
        }
    }
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

/// Collect all unique helper names from conditions.
fn collect_helpers(children: &[Ast]) -> Vec<SetupHook> {
    let mut helpers = HashSet::new();
    collect_helpers_recursive(children, &mut helpers);
    let mut sorted: Vec<SetupHook> = helpers.into_iter().collect();
    sorted.sort();
    sorted
}

/// Recursively collect helper names.
fn collect_helpers_recursive(
    children: &[Ast],
    helpers: &mut HashSet<SetupHook>,
) {
    for child in children {
        match child {
            Ast::Condition(condition) => {
                // only produce helpers for a branch if any of its children is also a branch, meaning
                // there's a potential need to reuse them
                if condition.children.iter().any(|c| match c {
                    Ast::Condition(_) => true,
                    _ => false,
                }) {
                    helpers.insert(SetupHook {
                        name: to_snake_case(&condition.title),
                    });
                }
                collect_helpers_recursive(&condition.children, helpers);
            }
            Ast::Root(root) => {
                collect_helpers_recursive(&root.children, helpers);
            }
            _ => {}
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
