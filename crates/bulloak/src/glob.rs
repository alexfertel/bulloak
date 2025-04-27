use std::path::PathBuf;

use glob::glob;

pub(crate) fn expand_glob(
    input: PathBuf,
) -> anyhow::Result<impl Iterator<Item = PathBuf>> {
    let input = input.to_string_lossy();
    let paths = glob(&input)?.filter_map(Result::ok);
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::expand_glob;

    /// Helper to collect and sort the output.
    fn sorted_matches(pattern: &str) -> Vec<String> {
        let mut v: Vec<_> = expand_glob(PathBuf::from(pattern))
            .unwrap()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
        v.sort();
        v
    }

    #[test]
    fn literal_path_round_trips() {
        // This crate has a Cargo.toml in its root.
        let out = sorted_matches("Cargo.toml");
        assert_eq!(out, vec!["Cargo.toml".to_string()]);
    }

    #[test]
    fn no_such_file_yields_empty() {
        let out = sorted_matches("no-such-file-xyz.tree");
        assert!(out.is_empty());
    }

    #[test]
    fn simple_star_glob() {
        // Match all .rs files in src/
        let out = sorted_matches("src/*.rs");
        assert!(out.iter().any(|e| e.ends_with("main.rs")));
        assert!(out.iter().any(|e| e.ends_with("check.rs")));
    }

    #[test]
    fn recursive_double_star_glob() {
        // `tests` directory has .tree files under tests/scaffold/.
        let out = sorted_matches("tests/scaffold/**/*.tree");
        assert!(out.iter().any(|e| e.ends_with("basic.tree")));
        assert!(out.iter().any(|e| e.ends_with("complex.tree")));
    }

    #[test]
    fn forward_slash_glob_works_everywhere() {
        let out = sorted_matches("tests/scaffold/*.tree");
        assert!(out.iter().any(|e| e.ends_with("basic.tree")));
    }

    #[test]
    #[cfg(windows)]
    fn backslash_glob_works_the_same() {
        let fwd = sorted_matches("tests/scaffold/*.tree");
        let bwd = sorted_matches("tests\\scaffold\\*.tree");
        assert_eq!(
            fwd, bwd,
            "backslash‐based glob must match forward‐slash one"
        );
    }
}
