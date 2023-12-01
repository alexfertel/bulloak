//! Defines a rule-checking error object.

use std::fmt;

use forge_fmt::parse;
use forge_fmt::solang_ext::{CodeLocationExt, SafeUnwrap};
use owo_colors::OwoColorize;
use solang_parser::pt;
use solang_parser::pt::{ContractDefinition, ContractPart};
use std::collections::HashSet;

use crate::constants::INTERNAL_DEFAULT_INDENTATION;
use crate::constants::INTERNAL_DEFAULT_SOL_VERSION;
use crate::error;
use crate::hir::{self, Hir};
use crate::scaffold::emitter::Emitter;
use crate::sol::find_matching_fn;
use crate::sol::{self, find_contract};

use super::context::Context;
use super::location::Location;

/// An error that occurred while checking specification rules between
/// a tree and a Solidity contract.
#[derive(Debug)]
pub(crate) struct Violation {
    /// The kind of violation.
    pub(crate) kind: ViolationKind,
    pub(crate) location: Location,
}

impl Violation {
    /// Create a new violation.
    pub(crate) fn new(kind: ViolationKind, location: Location) -> Self {
        Self { kind, location }
    }

    /// Determines whether a given violation is fixable.
    pub(crate) fn is_fixable(&self) -> bool {
        return matches!(
            self.kind,
            ViolationKind::ContractMissing(_)
                | ViolationKind::ContractNameNotMatches(_, _)
                | ViolationKind::FunctionOrderMismatch(_, _, _)
                | ViolationKind::MatchingFunctionMissing(_, _)
        );
    }

    /// Optionally returns a help text to be used when displaying the violation kind.
    pub(crate) fn help(&self) -> Option<String> {
        match &self.kind {
            ViolationKind::ContractMissing(name) => {
                Some(format!(r#"consider adding a contract with name "{name}""#))
            }
            ViolationKind::ContractNameNotMatches(name, _) => {
                Some(format!(r#"consider renaming the contract to "{name}""#))
            }
            ViolationKind::SolidityFileMissing(filename) => {
                let filename = filename.replace(".t.sol", ".tree");
                Some(format!("consider running `bulloak scaffold {filename}`"))
            }
            ViolationKind::FunctionOrderMismatch(_, _, _) => {
                Some("consider reordering the function in the file".to_owned())
            }
            _ => None,
        }
    }
}

/// The type of an error that occurred while checking specification rules between
/// a tree and a Solidity contract.
///
/// NOTE: Adding a variant to this enum most certainly will mean adding a variant to the
/// `Rules` section of `bulloak`'s README. Please, do not forget to add it if you are
/// implementing a rule.
#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum ViolationKind {
    /// Found no matching Solidity contract.
    ///
    /// (contract name)
    ContractMissing(String),
    /// Contract name doesn't match.
    ///
    /// (tree name, sol name)
    ContractNameNotMatches(String, String),
    /// The corresponding Solidity file does not exist.
    SolidityFileMissing(String),
    /// Couldn't read the corresponding Solidity file.
    FileUnreadable,
    /// Found an incorrectly ordered element.
    ///
    /// (pt function, current position, insertion position)
    FunctionOrderMismatch(pt::FunctionDefinition, usize, usize),
    /// Found a tree element without its matching codegen.
    ///
    /// (hir function, insertion position)
    MatchingFunctionMissing(hir::FunctionDefinition, usize),
    /// The parsing of a tree or a Solidity file failed.
    ParsingFailed(anyhow::Error),
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let divider = repeat_str("•", 79);
        writeln!(f, "{divider}")?;

        writeln!(f, "check failed: {}", self.kind)?;
        writeln!(f, "{}", self.location)?;

        Ok(())
    }
}

impl fmt::Display for ViolationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::ViolationKind::{
            ContractMissing, ContractNameNotMatches, FileUnreadable, FunctionOrderMismatch,
            MatchingFunctionMissing, ParsingFailed, SolidityFileMissing,
        };
        match self {
            FileMissing(filename) => {
                write!(
                    f,
                    "the file is missing its matching Solidity file.\nTry running `bulloak scaffold {filename}`"
                )
            }
            FileUnreadable => {
                write!(f, "bulloak couldn't read the file")
            }
            ContractMissing(contract) => write!(
                f,
                r#"couldn't find a corresponding contract for "{contract}" in the Solidity file"#
            ),
            ContractNameNotMatches(tree_name, sol_name) => write!(
                f,
                r#"couldn't find a corresponding contract for "{tree_name}" in the Solidity file. Found "{sol_name}""#
            ),
            FunctionOrderMismatch(fn_sol, _, _) => {
                let name = fn_sol.name.safe_unwrap();
                write!(f, r#"incorrect position for function `{name}`"#)
            }
            MatchingFunctionMissing(fn_hir, _) => {
                let name = fn_hir.identifier.clone();
                write!(f, r#"function "{name}" is missing in .sol"#)
            }
            ParsingFailed(error) => {
                if let Some(error) = error.downcast_ref::<error::Error>() {
                    match error {
                        error::Error::Tokenize(error) => write!(
                            f,
                            "an error occurred while parsing the tree: {}",
                            error.kind()
                        ),
                        error::Error::Parse(error) => write!(
                            f,
                            "an error occurred while parsing the tree: {}",
                            error.kind()
                        ),
                        error::Error::Semantic(_) => write!(
                            f,
                            "at least one semantic error occured while parsing the tree"
                        ),
                    }
                } else {
                    write!(f, "an error occurred while parsing the solidity file")
                }
            }
        }
    }
}

impl ViolationKind {
    pub(crate) fn fix(&self, mut ctx: Context) -> Context {
        match self {
            ViolationKind::ContractMissing(_) => {
                let pt = sol::Translator::new(INTERNAL_DEFAULT_SOL_VERSION).translate(&ctx.hir);
                let source = sol::Formatter::new().emit(pt.clone());
                let parsed = parse(&source).expect("should parse solidity string");
                ctx.from_parsed(parsed)
            }
            ViolationKind::ContractNameNotMatches(new_name, old_name) => {
                let source = ctx.src.replace(
                    &format!("contract {old_name}"),
                    &format!("contract {new_name}"),
                );
                let parsed = parse(&source).expect("should parse solidity string");
                ctx.from_parsed(parsed)
            }
            // Assume order violations have been taken care of first.
            ViolationKind::MatchingFunctionMissing(fn_hir, index) => {
                if let Some(contract_hir) = ctx.hir.find_contract() {
                    if let Some(contract_sol) = find_contract(&ctx.pt) {
                        if let Some(offset) =
                            get_insertion_offset(&contract_sol, contract_hir, *index, &ctx)
                        {
                            ctx.src = insert_function(fn_hir, &ctx.src, offset);
                        }

                        let source = ctx.src.clone();
                        let parsed = parse(&source).expect("should parse solidity string");
                        return ctx.from_parsed(parsed);
                    }
                }

                ctx
            }
            _ => ctx,
        }
    }
}

/// Inserts a function definition into a source string at a specified offset.
///
/// This function takes a `FunctionDefinition` from the High-Level Intermediate Representation (HIR),
/// converts it into a Solidity function definition string using an `Emitter`, and then inserts
/// this string into the specified source code at a given offset.
///
/// # Arguments
/// * `function` - A reference to the HIR `FunctionDefinition` to be inserted.
/// * `src` - The source string into which the function definition will be inserted.
/// * `offset` - The character position in the source string where the function definition should be inserted.
///
/// # Returns
/// A new `String` containing the source with the function definition inserted at the specified offset.
fn insert_function(function: &hir::FunctionDefinition, src: &str, offset: usize) -> String {
    let function = Emitter::new(INTERNAL_DEFAULT_INDENTATION, INTERNAL_DEFAULT_SOL_VERSION)
        .emit(&Hir::FunctionDefinition(function.clone()));
    format!(
        "{}\n\n{}{}",
        &src[..offset],
        function.trim_end(),
        &src[offset..]
    )
}

/// Determines the appropriate insertion offset for a function within a contract source code.
///
/// This function calculates the character position in the source code at which a new function
/// should be inserted. If the function to be inserted is the first one (`index` is 0), the
/// offset is calculated as the position right after the opening brace of the contract. Otherwise,
/// it finds the location immediately after the function that precedes the insertion point in the
/// HIR structure.
///
/// # Arguments
/// * `contract_sol` - A `ContractDefinition` from the Solidity parse tree.
/// * `contract_hir` - A `ContractDefinition` in the HIR node corresponding to the contract.
/// * `index` - The index at which the function is to be inserted in the contract children.
/// * `ctx` - A reference to the `Context` containing additional information, including the source code.
///
/// # Returns
/// An `Option<usize>` representing the calculated offset position where the function should be inserted.
/// Returns `None` if the offset cannot be determined, such as when a preceding function cannot be found.
///
/// # Panics
/// Panics if it fails to locate the opening brace of the contract while processing the first function
/// (`index` is 0), indicating an issue with the solidity program structure.
fn get_insertion_offset(
    contract_sol: &pt::ContractDefinition,
    contract_hir: &hir::ContractDefinition,
    index: usize,
    ctx: &Context,
) -> Option<usize> {
    if index == 0 {
        let contract_start = contract_sol.loc.start();
        let opening_brace_pos = ctx
            .src
            .chars()
            .skip(contract_start)
            .position(|c| c == '{')
            .expect("should search over a valid solidity program");

        let offset = contract_start + opening_brace_pos + 1;
        return Some(offset);
    } else if let Hir::FunctionDefinition(ref pre_fn_hir) = contract_hir.children[index - 1] {
        let prev_fn = find_matching_fn(&contract_sol, pre_fn_hir);
        if let Some((_, prev_fn)) = prev_fn {
            let offset = prev_fn.loc().end();
            return Some(offset);
        }

        unreachable!()
    };

    None
}

/// `fix_order` rearranges the functions in a Solidity contract (`contract_sol`) to match the order
/// specified in a higher-level intermediate representation (`contract_hir`).
///
/// # Arguments
/// * `violations`: A slice of `Violation` instances. Each `Violation` represents a discrepancy
///   between the order of functions in the Solidity contract and the higher-level intermediate
///   representation.
/// * `contract_sol`: A reference to a `Box<ContractDefinition>`, representing the Solidity contract
///   whose function order needs correction.
/// * `contract_hir`: A reference to a `hir::ContractDefinition`, representing the higher-level
///   intermediate representation that dictates the correct order of functions.
/// * `ctx`: The current `Context` which holds the source code and other relevant data for
///   processing the contract.
///
/// # Returns
/// Returns a `Context` instance, which is an updated version of the input `ctx`. This updated
/// `Context` contains the Solidity source code with the functions reordered as per the order
/// specified in `contract_hir`.
///
/// # Process
/// The function works in several steps:
/// 1. It first creates a set of function names present in `contract_hir` to identify the functions
///    that need ordering.
/// 2. It then iterates over the `violations` to correct the order of functions in `contract_sol`,
///    matching them with `contract_hir`.
/// 3. Functions not part of `contract_hir` are removed, as their correct position is unknown.
/// 4. The sorted functions are then compiled into a string (`source`) and simultaneously removed
///    from a temporary string (`scratch`) that mirrors the original source code.
/// 5. Finally, the function reconstructs the contract's body by combining the sorted functions
///    and any remaining parts of the contract (preserved in `scratch`), ensuring all components
///    are included in the output.
///
/// # Panics
/// The function will panic if the reconstructed Solidity string fails to parse. This is a safeguard
/// to ensure the output is always a valid Solidity code.
pub(crate) fn fix_order(
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
            if let hir::Hir::FunctionDefinition(f) = &child {
                Some(f.identifier.clone())
            } else {
                None
            }
        })
        .collect();

    // 2. Properly sort functions in a new vec.
    let mut fns = contract_sol.parts.clone();
    for violation in violations {
        if let ViolationKind::FunctionOrderMismatch(f, sol_idx, hir_idx) = &violation.kind {
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

    // 4. a - Add the functions that appear in the tree to a blank
    // string `source`.
    //    b - Replace them with whitespace in `scratch`.
    //
    // Since we sorted in a previos step, they'll appear sorted in
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
    let contract_body_len = contract_sol.loc().end() - contract_sol.loc().start();
    // We know there is at least two parts because we found order violations.
    let first_part_loc = contract_sol.parts[0].loc();
    // If the functions in the solidity file are exactly the functions in the
    // tree file, then we just print them. We still need to include the scratch
    // because it might contain comments or other constructs that we need to keep.
    let source = if fns.len() == contract_sol.parts.len() {
        format!(
            "{}{}{}{}",
            &ctx.src[..first_part_loc.start()],
            source.join("\n\n"),
            &scratch[first_part_loc.start()..contract_body_len],
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
            &scratch[first_part_loc.start()..contract_body_len],
            &ctx.src[contract_sol.loc().end() - 1..]
        )
    };

    let parsed = parse(&source).expect("should parse solidity string");
    ctx.from_parsed(parsed)
}
