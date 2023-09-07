use std::{
    env, fs,
    path::PathBuf,
    process::{Command, Output},
};

use pretty_assertions::assert_eq;

fn scaffold(tree_path: &PathBuf, binary_path: &PathBuf) -> Output {
    Command::new(binary_path)
        .arg("scaffold")
        .arg(tree_path)
        .output()
        .expect("should execute the scaffold command")
}

#[test]
fn test_scaffold_trees() {
    let cwd = env::current_dir().unwrap();
    let trees = fs::read_dir(cwd.join("tests/trees")).expect("should read the trees directory");
    let trees: Vec<PathBuf> = trees.filter_map(|entry| Some(entry.ok()?.path())).collect();

    let binary_path = cwd.join("target/debug/bulloak");

    for tree_path in trees {
        if tree_path.extension().unwrap() == "tree" {
            let output = scaffold(&tree_path, &binary_path);
            let actual = String::from_utf8(output.stdout).unwrap();

            let mut output_file = tree_path.clone();
            output_file.set_extension("sol");
            let expected = fs::read_to_string(output_file).unwrap();

            // We trim here because we don't care about ending newlines.
            assert_eq!(expected.trim(), actual.trim());
        }
    }
}
