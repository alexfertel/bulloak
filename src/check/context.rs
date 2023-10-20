//! Defines the context in which rule-checking occurs.

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use crate::check::Violation;
use solang_parser::pt::SourceUnit;

use crate::hir::Hir;

use super::{location::Location, violation::ViolationKind};

/// The context in which rule-checking happens.
///
/// This is a utility struct that abstracts away the requirements
/// for a `check` call. If you need any additional information
/// for your rule, feel free to add it here.
#[derive(Clone, Debug)]
pub(crate) struct Context {
    /// The path to the tree file.
    pub(crate) tree_path: PathBuf,
    /// The high-level intermediate representation
    /// of the bulloak tree.
    pub(crate) tree_hir: Hir,
    /// The path to the Solidity file.
    pub(crate) sol_path: PathBuf,
    /// The contents of the Solidity file.
    pub(crate) sol_contents: String,
    /// The abstract syntax tree of the Solidity file.
    pub(crate) sol_ast: SourceUnit,
}

impl Context {
    pub(crate) fn new(tree_path: PathBuf) -> Result<Self, Violation> {
        let tree_path_cow = tree_path.to_string_lossy();
        let tree_contents = try_read_to_string(&tree_path)?;
        let tree_hir = crate::hir::translate(&tree_contents).map_err(|e| {
            Violation::new(
                ViolationKind::ParsingFailed(e),
                Location::File(tree_path_cow.into_owned()),
            )
        })?;

        let sol_path = get_path_with_ext(&tree_path, "t.sol")?;
        let sol_contents = try_read_to_string(&sol_path)?;
        let (sol_ast, _) =
            solang_parser::parse(&sol_contents, 0).expect("should parse the Solidity file");

        Ok(Context {
            tree_path,
            tree_hir,
            sol_path,
            sol_ast,
            sol_contents,
        })
    }
}

fn get_path_with_ext<P, S>(path: P, ext: S) -> Result<PathBuf, Violation>
where
    P: AsRef<Path>,
    S: AsRef<OsStr>,
{
    let path = path.as_ref();
    let mut sol_path = path.to_path_buf();
    sol_path.set_extension(ext);

    if !sol_path.exists() {
        let filename = path.to_string_lossy().into_owned();
        return Err(Violation::new(
            ViolationKind::FileMissing(filename.clone()),
            Location::File(filename),
        ));
    }

    Ok(sol_path)
}

fn try_read_to_string<P>(path: P) -> Result<String, Violation>
where
    P: AsRef<Path>,
{
    fs::read_to_string(&path).map_err(|_| {
        let path = path.as_ref().to_string_lossy();
        Violation::new(
            ViolationKind::FileUnreadable,
            Location::File(path.into_owned()),
        )
    })
}
