use std::{
    env, fs,
    path::PathBuf,
    process::{Command, Output},
};

use pretty_assertions::assert_eq;

fn get_trees(cwd: &PathBuf) -> Vec<PathBuf> {
    let trees = fs::read_dir(cwd.join("tests/check")).expect("should read the check directory");
    let trees: Vec<PathBuf> = trees.filter_map(|entry| Some(entry.ok()?.path())).collect();
    trees
}

fn check(binary_path: &PathBuf, tree_path: &PathBuf, args: &[&str]) -> Output {
    Command::new(binary_path)
        .arg("check")
        .arg(tree_path)
        .args(args)
        .output()
        .expect("should execute the check command")
}

#[test]
fn checks_structural_match() {
    let cwd = env::current_dir().unwrap();
    let trees = get_trees(&cwd);
    let binary_path = cwd.join("target").join("debug").join("bulloak");

    for tree_path in trees {
        if tree_path.extension().unwrap() == "tree" {
            let output = check(&binary_path, &tree_path, &[]);
            let actual = String::from_utf8(output.stderr).unwrap();

            let expected = r#"Codegen not found: Couldn't find a corresponding element for "test_GivenTheSenderDoesNotImplementTheHook" in the solidity file.
Invalid codegen order: Found a matching element for "givenTheSenderImplementsTheHook", but the order is not correct.
Invalid codegen order: Found a matching element for "givenTheStreamsStatusIsCANCELED", but the order is not correct.
Invalid codegen order: Found a matching element for "givenTheStreamsStatusIsSETTLED", but the order is not correct.
Invalid codegen order: Found a matching element for "test_RevertGiven_TheStreamsStatusIsCANCELED", but the order is not correct.
Invalid codegen order: Found a matching element for "test_RevertGiven_TheStreamsStatusIsDEPLETED", but the order is not correct.
Invalid codegen order: Found a matching element for "test_RevertGiven_TheStreamsStatusIsSETTLED", but the order is not correct.
Invalid codegen order: Found a matching element for "whenTheSenderReverts", but the order is not correct."#;

            // We trim here because we don't care about ending newlines.
            assert_eq!(expected.trim(), actual.trim());
        }
    }
}
