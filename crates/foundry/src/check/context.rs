//! Defines the context in which rule-checking occurs.

use std::{
    collections::HashSet,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use solang_forge_fmt::{
    format, parse,
    solang_ext::{CodeLocationExt, SafeUnwrap},
    Comments, FormatterError, Parsed,
};
use solang_parser::pt::{self, ContractDefinition, ContractPart, SourceUnit};

use super::{location::Location, violation::ViolationKind};
use crate::{
    check::{pretty::Pretty, violation::Violation},
    config::Config,
    hir::{self, Hir},
    scaffold::emitter::Emitter,
    sol::{self, find_contract, find_matching_fn},
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
        let hir = crate::hir::translate(&tree_contents, cfg).map_err(|e| {
            Violation::new(
                ViolationKind::ParsingFailed(e),
                Location::File(tree_path_cow.into_owned()),
            )
        })?;

        let sol = get_path_with_ext(&tree, "t.sol")?;
        let src = try_read_to_string(&sol)?;
        let parsed = solang_forge_fmt::parse(&src).map_err(|_| {
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
    pub fn update_from_parsed(mut self, parsed: Parsed) -> Self {
        parsed.src.clone_into(&mut self.src);
        self.pt = parsed.pt;
        self.comments = parsed.comments;
        self
    }

    /// Updates the context with a formatted representation of the Solidity
    /// file.
    pub fn fmt(self) -> anyhow::Result<String, FormatterError> {
        format(&self.src)
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
        let f = &Hir::Function(function.clone());
        let function = Emitter::new(&self.cfg).emit(f);
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

impl Context {
    pub(crate) fn fix_contract_missing(self) -> anyhow::Result<Context> {
        let pt = sol::Translator::new(&self.cfg).translate(&self.hir);
        let source = sol::Formatter::new().emit(pt.clone());
        let filename = self.sol.to_string_lossy();
        let parsed = parse(&source).map_err(|diagnostics| {
            let full = diagnostics
                .into_iter()
                .map(|d| Pretty::new(&d, &filename, &source).to_string())
                .collect::<Vec<_>>()
                .join("\n");
            anyhow::anyhow!(full)
        })?;
        Ok(self.update_from_parsed(parsed))
    }

    pub(crate) fn fix_contract_rename(
        self,
        new_name: &str,
        old_name: &str,
    ) -> anyhow::Result<Context> {
        let source = self.src.replace(
            &format!("contract {old_name}"),
            &format!("contract {new_name}"),
        );
        let filename = self.sol.to_string_lossy();
        let parsed = parse(&source).map_err(|diagnostics| {
            let full = diagnostics
                .into_iter()
                .map(|d| Pretty::new(&d, &filename, &source).to_string())
                .collect::<Vec<_>>()
                .join("\n");
            anyhow::anyhow!(full)
        })?;
        Ok(self.update_from_parsed(parsed))
    }

    pub(crate) fn fix_matching_fn_missing(
        mut self,
        fn_hir: &hir::FunctionDefinition,
        index: usize,
    ) -> anyhow::Result<Context> {
        let contract_hir = match self.hir.find_contract() {
            Some(c) => c,
            None => return Ok(self),
        };
        let contract_sol = match find_contract(&self.pt) {
            Some(c) => c,
            None => return Ok(self),
        };

        let offset =
            get_insertion_offset(&contract_sol, contract_hir, index, &self.src);
        self.insert_function_at(fn_hir, offset);

        let source = self.src.clone();
        let filename = self.sol.to_string_lossy();
        let parsed = parse(&source).map_err(|diagnostics| {
            let full = diagnostics
                .into_iter()
                .map(|d| Pretty::new(&d, &filename, &source).to_string())
                .collect::<Vec<_>>()
                .join("\n");
            anyhow::anyhow!(full)
        })?;
        Ok(self.update_from_parsed(parsed))
    }
}

/// Calculates the insertion offset for a new function in a contract's source
/// code.
///
/// # Arguments
/// * `contract_sol` - Solidity parse tree contract definition
/// * `contract_hir` - HIR contract definition
/// * `index` - Insertion index for the new function
/// * `src` - Source code reference
///
/// # Returns
/// Offset position for function insertion
///
/// # Panics
/// If the contract's opening brace cannot be located when processing the first
/// function
fn get_insertion_offset(
    contract_sol: &pt::ContractDefinition,
    contract_hir: &hir::ContractDefinition,
    index: usize,
    src: impl AsRef<str>,
) -> usize {
    if index == 0 {
        return find_contract_body_start(contract_sol, src.as_ref());
    }

    match &contract_hir.children[index - 1] {
        Hir::Function(prev_fn_hir) => {
            find_matching_fn(contract_sol, prev_fn_hir)
                .expect("matching function should exist")
                .1
                .loc()
                .end()
        }
        _ => unreachable!("previous child should be a function definition"),
    }
}

/// Finds the starting position of a contract's body in the source code.
///
/// # Arguments
/// * `contract_sol` - Solidity parse tree contract definition
/// * `src` - Full source code string
///
/// # Returns
/// `Result` with the position immediately after the opening brace, or an error
/// if not found
fn find_contract_body_start(
    contract_sol: &pt::ContractDefinition,
    src: &str,
) -> usize {
    let contract_start = contract_sol.loc.start();
    let opening_brace_pos = src[contract_start..]
        .find('{')
        .expect("contract should have an opening brace");

    contract_start + opening_brace_pos + 1
}

/// Rearranges functions in a Solidity contract to match the order in the HIR.
///
/// The algorithm goes like this:
/// 1. It first creates a set of function names present in `contract_hir` to
///    identify the functions that need to be sorted.
/// 2. It then iterates over the `violations` to correct the order of functions
///    in `contract_sol`, matching them with `contract_hir`.
/// 3. Functions not part of `contract_hir` are removed, as their correct
///    position is unknown.
/// 4. The sorted functions are then compiled into a string (`source`) and
///    simultaneously removed from a temporary string (`scratch`) that mirrors
///    the original source code.
/// 5. Finally, the function reconstructs the contract's body by combining the
///    sorted functions and any remaining parts of the contract (preserved in
///    `scratch`), ensuring all components are included in the output.
///
/// # Arguments
/// * `violations` - Order discrepancies between Solidity and HIR
/// * `contract_sol` - Solidity contract to be corrected
/// * `contract_hir` - HIR contract with correct function order
/// * `ctx` - Current context with source code and processing data
///
/// # Returns
/// Updated Context with reordered functions in Solidity source code
///
/// # Panics
/// If the reconstructed Solidity string fails to parse
#[must_use]
pub fn fix_order(
    violations: &[Violation],
    contract_sol: &Box<ContractDefinition>,
    contract_hir: &hir::ContractDefinition,
    ctx: Context,
) -> Context {
    // 1. Create a set containing the functions that appear in the tree.
    //
    // These are the functions that we know how to sort.
    let fn_names: HashSet<String> = contract_hir
        .children
        .iter()
        .filter_map(|child| {
            if let hir::Hir::Function(f) = &child {
                Some(f.identifier.clone())
            } else {
                None
            }
        })
        .collect();

    // 2. Properly sort functions in a new vec.
    let mut fns = contract_sol.parts.clone();
    for violation in violations {
        if let ViolationKind::FunctionOrderMismatch(f, sol_idx, hir_idx) =
            &violation.kind
        {
            fns.remove(*sol_idx);
            fns.insert(
                *hir_idx,
                ContractPart::FunctionDefinition(Box::new(f.clone())),
            );
        }
    }

    // 3. Remove functions that are not part of the hir since
    // we don't know where they go.
    let fns: Vec<&ContractPart> = fns
        .iter()
        .filter(|f| {
            if let ContractPart::FunctionDefinition(f) = f {
                fn_names.contains(&f.name.safe_unwrap().name)
            } else {
                false
            }
        })
        .collect();

    // 4. a - Add the functions that appear in the tree to the blank
    // string `source`.
    //    b - Replace them with whitespace in `scratch`.
    //
    // Since we sorted in a previous step, they'll appear sorted in
    // the string. We do 4.b because we want to append the remaining
    // functions after the sorted functions.
    let mut scratch = ctx.src.clone();
    let mut source = Vec::with_capacity(fns.len());
    for f in &fns {
        let loc = f.loc();
        let f = ctx.src[loc.start()..loc.end()].to_owned();
        scratch = scratch.replace(&f, " ".repeat(f.len()).as_str());
        source.push(f);
    }

    // 5. Replace the contract's body with the sorted functions and
    // the extra functions contained in the scratch string.
    // We know there is at least two parts because we found order violations.
    let first_part_loc = contract_sol.parts[0].loc();
    // If the functions in the solidity file are exactly the functions in the
    // tree file, then we just print them. We still need to include the scratch
    // because it might contain comments or other constructs that we need to
    // keep.
    let source = if fns.len() == contract_sol.parts.len() {
        format!(
            "{}{}{}{}",
            &ctx.src[..first_part_loc.start()],
            source.join("\n\n"),
            &scratch[first_part_loc.start()..contract_sol.loc.end() - 1],
            &ctx.src[contract_sol.loc().end() - 1..]
        )
    } else {
        const SEPARATOR: &str = r"

      // <<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<
      // ==================== BULLOAK AUTOGENERATED SEPARATOR ====================
      // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
      //    Code below this section could not be automatically moved by bulloak
      // =========================================================================
            ";
        format!(
            "{}{}{SEPARATOR}{}{}",
            &ctx.src[..first_part_loc.start()],
            source.join("\n\n"),
            &scratch[first_part_loc.start()..contract_sol.loc.end() - 1],
            &ctx.src[contract_sol.loc().end() - 1..]
        )
    };

    let parsed = parse(&source).expect("should parse solidity string");
    ctx.update_from_parsed(parsed)
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write};

    use tempfile::tempdir;

    use super::*;
    use crate::check::rules::{Checker, StructuralMatcher};

    fn write_file(
        dir: &std::path::Path,
        name: &str,
        contents: &str,
    ) -> std::path::PathBuf {
        let path = dir.join(name);
        let mut f = fs::File::create(&path).unwrap();
        write!(f, "{}", contents).unwrap();
        path
    }

    fn make_ctx(tree: &str, sol: &str) -> Context {
        let td = tempdir().unwrap();
        let tree_path = write_file(td.path(), "X.tree", tree);
        let sol_path = td.path().join("X.t.sol");
        fs::write(&sol_path, sol).unwrap();
        let mut cfg = Config::default();
        cfg.files = vec![tree_path.clone()];
        Context::new(tree_path, &cfg).unwrap()
    }

    #[test]
    fn fix_contract_rename_updates_contract_name() {
        let tree = "Foo\n└── It one.\n";
        let sol = "\
            // SPDX-License-Identifier: UNLICENSED\n\
            pragma solidity 0.8.0;\n\
            contract Bar {}\n";
        let ctx0 = make_ctx(tree, sol);
        let ctx1 = ctx0.clone().fix_contract_rename("Foo", "Bar").unwrap();
        assert!(ctx1.src.contains("contract Foo"));
        assert!(!ctx1.src.contains("contract Bar"));
    }

    #[test]
    fn fix_matching_fn_missing_inserts_test_stub() {
        let tree = "Foo\n└── It one.\n";
        let sol = "\
            // SPDX-License-Identifier: UNLICENSED\n\
            pragma solidity 0.8.0;\n\
            contract Foo {}\n";
        let ctx0 = make_ctx(tree, sol);
        let mut vs = StructuralMatcher::check(&ctx0);
        assert_eq!(1, vs.len());
        let ctx1 = vs.pop().unwrap().kind.fix(ctx0).unwrap();
        assert!(
            ctx1.src.contains("function test_One()"),
            "expected inserted test_One stub"
        );
    }

    #[test]
    fn fix_contract_missing_scaffolds_tests() {
        let tree = "Foo\n└── It one.\n";
        let sol = "\
            // SPDX-License-Identifier: UNLICENSE-Identifier\n\
            pragma solidity 0.8.0;\n\
            contract Foo {}\n";
        let ctx0 = make_ctx(tree, sol);
        let ctx1 = ctx0.clone().fix_contract_missing().unwrap();
        assert!(
            ctx1.src.contains("function test_One()"),
            "should scaffold the missing test"
        );
    }

    #[test]
    fn fix_order_reorders_functions() {
        let tree = "\
            Foo\n\
            ├── It A.\n\
            ├── It B.\n\
            └── It C.\n";
        let sol = "\
            // SPDX-License-Identifier: UNLICENSE-Identifier\n\
            pragma solidity 0.8.0;\n\
            contract Foo {\n\
              function test_B() external {}\n\
              function test_C() external {}\n\
              function test_A() external {}\n\
            }\n";
        let ctx0 = make_ctx(tree, sol);
        let vs = StructuralMatcher::check(&ctx0);
        let contract_hir = ctx0.hir.find_contract().unwrap();
        let contract_sol = crate::sol::find_contract(&ctx0.pt).unwrap();
        let ctx1 = fix_order(&vs, &contract_sol, contract_hir, ctx0.clone());
        // after fix, A,B,C in that order
        let src = &ctx1.src;
        let idx_a = src.find("test_A").unwrap();
        let idx_b = src.find("test_B").unwrap();
        let idx_c = src.find("test_C").unwrap();
        assert!(
            idx_a < idx_b && idx_b < idx_c,
            "functions should be A,B,C in order"
        );
    }

    #[test]
    fn fix_rename_parse_error() {
        let tree = "Foo\n└── It one.\n";
        let sol = "\
            // SPDX-License-Identifier: UNLICENSED\n\
            pragma solidity 0.8.0;\n\
            contract Foo {}\n";
        let mut ctx = make_ctx(tree, sol);
        ctx.src = "contract Foo { invalid }".to_string();
        let err = ctx.clone().fix_contract_rename("Foo", "Bar").unwrap_err();
        assert!(err.to_string().to_lowercase().contains("error"));
    }

    #[test]
    fn fix_contract_missing_parse_error() {
        let tree = "Foo\n└── It one.\n";
        let sol = "\
            // SPDX-License-Identifier: UNLICENSE-Identifier\n\
            pragma solidity 0.8.0;\n\
            contract Foo {}\n";
        let mut ctx = make_ctx(tree, sol);
        ctx.src = "not a valid solidity file".to_string();
        let ctx1 = ctx.clone().fix_contract_missing().unwrap();
        assert!(ctx1.src.contains("function test_One()"));
    }

    #[test]
    fn fix_matching_fn_missing_no_contract() {
        let tree = "Foo\n└── It one.\n";
        let sol = "";
        let ctx0 = make_ctx(tree, sol);
        let contract_hir = ctx0.hir.find_contract().unwrap();
        if let Hir::Function(fn_hir) = &contract_hir.children[0] {
            let ctx1 =
                ctx0.clone().fix_matching_fn_missing(&fn_hir, 0).unwrap();
            assert_eq!(ctx0.src, ctx1.src);
        } else {
            unreachable!()
        };
    }

    #[test]
    fn fix_unfixable_variant_no_op() {
        let tree = "Foo\n└── It one.\n";
        let sol = "\
            // SPDX-License-Identifier: UNLICENSE-Identifier\n\
            pragma solidity 0.8.0;\n\
            contract Foo { \n\
              function test_One() external {} \n\
            }\n";
        let ctx0 = make_ctx(tree, sol);
        let contract_sol = find_contract(&ctx0.pt).unwrap();
        let fn_sol = contract_sol
            .parts
            .iter()
            .find_map(|p| {
                if let ContractPart::FunctionDefinition(f) = p {
                    Some((**f).clone())
                } else {
                    None
                }
            })
            .unwrap();
        let ctx1 = ViolationKind::FunctionOrderMismatch(fn_sol, 0, 1)
            .fix(ctx0.clone())
            .unwrap();
        assert_eq!(ctx0.src, ctx1.src);
    }

    #[test]
    fn context_fmt_roundtrip() {
        let tree = "Foo\n└── It one.\n";
        let sol = "\
            // SPDX-License-Identifier: UNLICENSE-Identifier\n\
            pragma solidity 0.8.0;\n\
            contract Foo {}\n";
        let ctx = make_ctx(tree, sol);
        let output = ctx.clone().fmt().unwrap();
        assert!(output.contains("contract Foo"));
        assert!(output.starts_with("// SPDX-License-Identifier"));
    }

    #[test]
    fn violation_display_with_help() {
        let violation = Violation::new(
            ViolationKind::ContractMissing("Foo".to_string()),
            Location::File("foo.tree".to_string()),
        );
        let s = format!("{violation}");
        assert!(s.contains("contract \"Foo\" is missing in .sol"));
        assert!(s.contains("consider adding a contract with name \"Foo\""));
    }
}
