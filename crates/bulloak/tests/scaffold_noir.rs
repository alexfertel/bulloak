#![allow(missing_docs)]
use std::{env};

use common::{cmd, get_binary_path};

mod common;

#[test]
fn errors_when_tree_is_empty() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tests_path = cwd.join("tests").join("scaffold");
    let trees = ["empty.tree"];

    for tree_name in trees {
        let tree_path = tests_path.join(tree_name);
        let output = cmd(&binary_path, "scaffold", &tree_path, &["-l", "noir"]);
        let actual = String::from_utf8(output.stderr).unwrap();

        assert!(output.status.code().unwrap() == 1);
        assert!(actual.contains("found an empty tree"));
    }
}
