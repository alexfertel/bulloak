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
        // this crate has a Cargo.toml in its root
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
        // match all .rs files in src/
        let out = sorted_matches("src/*.rs");
        // at least main.rs must be there
        assert!(out.iter().any(|e| e.ends_with("src/main.rs")));
        // and check.rs too
        assert!(out.iter().any(|e| e.ends_with("src/check.rs")));
    }

    #[test]
    fn recursive_double_star_glob() {
        // In your tests directory you have .tree files under tests/scaffold/
        let out = sorted_matches("tests/scaffold/**/*.tree");
        // A few smoke checks:
        assert!(out.iter().any(|e| e.contains("tests/scaffold/basic.tree")));
        assert!(out.iter().any(|e| e.contains("tests/scaffold/complex.tree")));
    }

    #[test]
    fn forward_slash_glob_works_everywhere() {
        let out = sorted_matches("tests/scaffold/*.tree");
        assert!(out.iter().any(|e| e.ends_with("tests/scaffold/basic.tree")));
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
