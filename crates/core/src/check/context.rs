//! Defines the context in which rule-checking occurs.

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use forge_fmt::{
    format, Comments, FormatterConfig, FormatterError, InlineConfig, Parsed,
};
use solang_parser::pt::SourceUnit;

use super::{location::Location, violation::ViolationKind};
use crate::{
    check::violation::Violation,
    config::Config,
    hir::{self, Hir},
    scaffold::emitter::Emitter,
};

/// The context in which rule-checking happens.
///
/// This is a utility struct that abstracts away the requirements for a `check`
/// call.
///
/// If you need any additional information for your rule, feel free to add it
/// here.
#[derive(Clone, Debug)]
pub struct Context {
    /// The path to the tree file.
    pub tree: PathBuf,
    /// The high-level intermediate representation
    /// of the bulloak tree.
    pub hir: Hir,
    /// The path to the Solidity file.
    pub sol: PathBuf,
    /// The contents of the Solidity file.
    pub src: String,
    /// The abstract syntax tree of the Solidity file.
    pub pt: SourceUnit,
    /// The comments present in the Solidity file.
    pub comments: Comments,
    /// The config passed to `bulloak check`.
    pub cfg: Config,
}

impl Context {
    /// Creates a new `Context`.
    ///
    /// This structure contains everything necessary to perform checks between
    /// trees and Solidity files.
    pub fn new(tree: PathBuf, cfg: &Config) -> Result<Self, Violation> {
        let tree_path_cow = tree.to_string_lossy();
        let tree_contents = try_read_to_string(&tree)?;
        let hir = crate::hir::translate(&tree_contents, &Config::default())
            .map_err(|e| {
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
                ViolationKind::ParsingFailed(anyhow::anyhow!(
                    "Failed to parse {sol_filename}"
                )),
                Location::File(sol_filename),
            )
        })?;

        let pt = parsed.pt.clone();
        let comments = parsed.comments;
        Ok(Context { tree, hir, sol, src, pt, comments, cfg: cfg.clone() })
    }

    /// Updates this `Context` with the result of parsing a Solidity file.
    #[inline]
    pub fn from_parsed(mut self, parsed: Parsed) -> Self {
        parsed.src.clone_into(&mut self.src);
        self.pt = parsed.pt;
        self.comments = parsed.comments;
        self
    }

    /// Updates the context with a formatted representation of the Solidity
    /// file.
    pub fn fmt(self) -> anyhow::Result<String, FormatterError> {
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

    /// Inserts a function definition into the source string at a specified
    /// offset.
    ///
    /// This function takes a `FunctionDefinition` from the High-Level
    /// Intermediate Representation (HIR), converts it into a Solidity
    /// function definition string using an `Emitter`, and then inserts this
    /// string into the specified source code at a given offset.
    ///
    /// # Arguments
    /// * `function` - A reference to the HIR `FunctionDefinition` to be
    ///   inserted.
    /// * `offset` - The character position in the source string where the
    ///   function definition should be inserted.
    pub fn insert_function_at(
        &mut self,
        function: &hir::FunctionDefinition,
        offset: usize,
    ) {
        let cfg = &Config::default();
        let f = &Hir::FunctionDefinition(function.clone());
        let function = Emitter::new(cfg).emit(f);
        self.src = format!(
            "{}\n\n{}{}",
            &self.src[..offset],
            function.trim_end(),
            &self.src[offset..]
        );
    }
}

fn get_path_with_ext(
    path: impl AsRef<Path>,
    ext: impl AsRef<OsStr>,
) -> Result<PathBuf, Violation> {
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

fn try_read_to_string(path: impl AsRef<Path>) -> Result<String, Violation> {
    fs::read_to_string(&path).map_err(|_| {
        let path = path.as_ref().to_string_lossy();
        Violation::new(
            ViolationKind::FileUnreadable,
            Location::File(path.into_owned()),
        )
    })
}
