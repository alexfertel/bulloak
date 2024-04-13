//! Defines the context in which rule-checking occurs.

use forge_fmt::{format, Comments, FormatterConfig, FormatterError, InlineConfig, Parsed};
use solang_parser::pt::SourceUnit;
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use crate::check::Violation;
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
    pub(crate) tree: PathBuf,
    /// The high-level intermediate representation
    /// of the bulloak tree.
    pub(crate) hir: Hir,
    /// The path to the Solidity file.
    pub(crate) sol: PathBuf,
    /// The contents of the Solidity file.
    pub(crate) src: String,
    /// The abstract syntax tree of the Solidity file.
    pub(crate) pt: SourceUnit,
    pub(crate) comments: Comments,
}

impl Context {
    pub(crate) fn new(tree: PathBuf) -> Result<Self, Violation> {
        let tree_path_cow = tree.to_string_lossy();
        let tree_contents = try_read_to_string(&tree)?;
        let hir = crate::hir::translate(&tree_contents, &false).map_err(|e| {
            Violation::new(
                ViolationKind::ParsingFailed(e),
                Location::File(tree_path_cow.into_owned()),
            )
        })?;

        let sol = get_path_with_ext(&tree, "t.sol")?;
        let src = try_read_to_string(&sol)?;
        let parsed = forge_fmt::parse(&src).map_err(|_| {
            let sol_filename = sol.to_string_lossy().into_owned();
            Violation::new(
                ViolationKind::ParsingFailed(anyhow::anyhow!("Failed to parse {sol_filename}")),
                Location::File(sol_filename),
            )
        })?;

        let pt = parsed.pt.clone();
        let comments = parsed.comments;
        Ok(Context {
            tree,
            hir,
            sol,
            src,
            pt,
            comments,
        })
    }

    #[inline]
    pub(crate) fn from_parsed(mut self, parsed: Parsed) -> Self {
        self.src = parsed.src.to_owned();
        self.pt = parsed.pt;
        self.comments = parsed.comments;
        self
    }

    pub(crate) fn fmt(self) -> anyhow::Result<String, FormatterError> {
        let mut formatted = String::new();
        format(
            &mut formatted,
            forge_fmt::Parsed {
                src: &self.src,
                pt: self.pt,
                comments: self.comments,
                inline_config: InlineConfig::default(),
                invalid_inline_config_items: Vec::default(),
            },
            FormatterConfig::default(),
        )?;

        Ok(formatted)
    }
}

fn get_path_with_ext<P, S>(path: P, ext: S) -> Result<PathBuf, Violation>
where
    P: AsRef<Path>,
    S: AsRef<OsStr>,
{
    let path = path.as_ref();
    let mut sol = path.to_path_buf();
    sol.set_extension(ext);

    if !sol.exists() {
        let filename = path.to_string_lossy().into_owned();
        return Err(Violation::new(
            ViolationKind::SolidityFileMissing(filename.clone()),
            Location::File(filename),
        ));
    }

    Ok(sol)
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
