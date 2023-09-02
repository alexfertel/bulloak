use std::{env, fs, io, path::PathBuf};

use pretty_assertions::assert_eq;

use bulloak;

fn scaffold(tree: &str) -> std::io::Result<String> {
    let scaffolder = bulloak::Scaffolder::new(true, 2, "0.8.0");

    let cwd = env::current_dir().unwrap();
    let tree_path = cwd.join("tests").join(tree);
    let text = fs::read_to_string(tree_path)?;
    Ok(scaffolder.scaffold(&text).expect("Should scaffold"))
}

fn get_trees(path: &str) -> io::Result<Vec<PathBuf>> {
    let entries = fs::read_dir(path)?;
    let all: Vec<PathBuf> = entries
        .filter_map(|entry| Some(entry.ok()?.path()))
        .collect();
    Ok(all)
}

#[test]
fn test_scaffold_trees() {
    let cwd = env::current_dir().unwrap();
    let trees_directory = cwd.join("tests").join("trees");
    for path in get_trees(trees_directory.to_str().unwrap()).expect("Should read trees") {
        if path.extension().unwrap() == "tree" {
            let actual = scaffold(path.to_str().unwrap()).unwrap();

            let mut output_file = path.clone();
            output_file.set_extension("sol");
            let expected = fs::read_to_string(output_file).unwrap();

            // We trim here because we don't care about ending newlines.
            assert_eq!(expected.trim(), actual.trim());
        }
    }
}
