use std::{
    env, fs,
    path::PathBuf,
    process::{Command, Output},
};

use owo_colors::OwoColorize;
use pretty_assertions::assert_eq;

fn get_trees(cwd: &PathBuf) -> Vec<PathBuf> {
    let trees = fs::read_dir(cwd.join("tests").join("scaffold"))
        .expect("should read the scaffold directory");
    let trees: Vec<PathBuf> = trees.filter_map(|entry| Some(entry.ok()?.path())).collect();
    trees
}

fn scaffold(binary_path: &PathBuf, tree_path: &PathBuf, args: &[&str]) -> Output {
    Command::new(binary_path)
        .arg("scaffold")
        .arg(tree_path)
        .args(args)
        .output()
        .expect("should execute the scaffold command")
}

#[test]
fn scaffolds_trees() {
    let cwd = env::current_dir().unwrap();
    let trees = get_trees(&cwd);
    let binary_path = cwd.join("target").join("debug").join("bulloak");

    for tree_path in trees {
        if tree_path.extension().unwrap() == "tree" {
            let output = scaffold(&binary_path, &tree_path, &[]);
            let actual = String::from_utf8(output.stdout).unwrap();

            let mut output_file = tree_path.clone();
            output_file.set_extension("t.sol");
            let expected = fs::read_to_string(output_file).unwrap();

            // We trim here because we don't care about ending newlines.
            assert_eq!(expected.trim(), actual.trim());
        }
    }
}

#[test]
fn skips_trees_when_file_exists() {
    let cwd = env::current_dir().unwrap();
    let trees = get_trees(&cwd);
    let binary_path = cwd.join("target").join("debug").join("bulloak");

    for tree_path in trees {
        if tree_path.extension().unwrap() == "tree" {
            let output = scaffold(&binary_path, &tree_path, &["-w"]);
            let actual = String::from_utf8(output.stderr).unwrap();

            let expected = format!("{}", "WARN".yellow());
            assert!(actual.starts_with(&expected));
        }
    }
}
