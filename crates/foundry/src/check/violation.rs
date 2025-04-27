//! Defines a rule-checking error object.
use std::{borrow::Cow, collections::HashSet, fmt};

use bulloak_syntax::FrontendError;
use forge_fmt::{
    parse,
    solang_ext::{CodeLocationExt, SafeUnwrap},
};
use owo_colors::OwoColorize;
use solang_parser::{
    pt,
    pt::{ContractDefinition, ContractPart},
};
use thiserror::Error;

use super::{context::Context, location::Location};
use crate::{
    check::pretty::Pretty,
    config::Config,
    hir::{self, Hir},
    sol::{self, find_contract, find_matching_fn},
};

/// An error that occurred while checking specification rules between
/// a tree and a Solidity contract.
#[derive(Debug, Error, PartialEq)]
pub struct Violation {
    /// The kind of violation.
    #[source]
    pub kind: ViolationKind,
    /// The location information about this violation.
    pub location: Location,
}

impl Violation {
    /// Create a new violation.
    pub fn new(kind: ViolationKind, location: Location) -> Self {
        Self { kind, location }
    }

    /// Determines whether a given violation is fixable.
    pub fn is_fixable(&self) -> bool {
        self.kind.is_fixable()
    }
}

/// The type of an error that occurred while checking specification rules
/// between a tree and a Solidity contract.
///
/// NOTE: Adding a variant to this enum most certainly will mean adding a
/// variant to the `Rules` section of `bulloak`'s README. Please, do not forget
/// to add it if you are implementing a rule.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ViolationKind {
    /// Found no matching Solidity contract.
    ///
    /// (contract name)
    #[error("contract \"{0}\" is missing in .sol")]
    ContractMissing(String),

    /// Contract name doesn't match.
    ///
    /// (tree name, sol name)
    #[error("contract \"{0}\" is missing in .sol -- found \"{1}\" instead")]
    ContractNameNotMatches(String, String),

    /// The corresponding Solidity file does not exist.
    #[error("the tree is missing its matching Solidity file: {0}")]
    SolidityFileMissing(String),

    /// Couldn't read the corresponding Solidity file.
    #[error("bulloak couldn't read the file")]
    FileUnreadable,

    /// Found an incorrectly ordered element.
    ///
    /// (pt function, current position, insertion position)
    #[error("incorrect position for function `{}`", .0.name.safe_unwrap())]
    FunctionOrderMismatch(pt::FunctionDefinition, usize, usize),

    /// Found a tree element without its matching codegen.
    ///
    /// (hir function, insertion position)
    #[error("function \"{}\" is missing in .sol", .0.identifier.clone())]
    MatchingFunctionMissing(hir::FunctionDefinition, usize),

    /// The parsing of a tree or a Solidity file failed.
    #[error("{}", format_frontend_error(.0))]
    ParsingFailed(#[from] anyhow::Error),
}

impl ViolationKind {
    /// Whether this violation kind is fixable.
    pub fn is_fixable(&self) -> bool {
        matches!(
            self,
            ViolationKind::ContractMissing(_)
                | ViolationKind::ContractNameNotMatches(_, _)
                | ViolationKind::FunctionOrderMismatch(_, _, _)
                | ViolationKind::MatchingFunctionMissing(_, _)
        )
    }

    /// Optionally returns a help text to be used when displaying the violation
    /// kind.
    pub fn help(&self) -> Option<Cow<'static, str>> {
        let text = match self {
            ViolationKind::ContractMissing(name) => {
                format!(r#"consider adding a contract with name "{name}""#)
                    .into()
            }
            ViolationKind::ContractNameNotMatches(name, _) => {
                format!(r#"consider renaming the contract to "{name}""#).into()
            }
            ViolationKind::SolidityFileMissing(filename) => {
                let filename = filename.replace(".t.sol", ".tree");
                format!("consider running `bulloak scaffold {filename}`").into()
            }
            ViolationKind::FunctionOrderMismatch(_, _, _) => {
                "consider reordering the function in the file".into()
            }
            _ => return None,
        };

        Some(text)
    }

    /// Returns a new context with this violation fixed.
    pub fn fix(&self, mut ctx: Context) -> anyhow::Result<Context> {
        Ok(match self {
            ViolationKind::ContractMissing(_) => {
                let pt = sol::Translator::new(&Config::default())
                    .translate(&ctx.hir);
                let source = sol::Formatter::new().emit(pt.clone());
                let filename = ctx.sol.to_string_lossy();
                let parsed = parse(&source).map_err(|diagnostics| {
                    let full_error = diagnostics
                        .into_iter()
                        .map(|d| {
                            Pretty::new(&d, &filename, &source).to_string()
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    anyhow::anyhow!("{}", full_error)
                })?;
                ctx.from_parsed(parsed)
            }
            ViolationKind::ContractNameNotMatches(new_name, old_name) => {
                let source = ctx.src.replace(
                    &format!("contract {old_name}"),
                    &format!("contract {new_name}"),
                );
                let filename = ctx.sol.to_string_lossy();
                let parsed = parse(&source).map_err(|diagnostics| {
                    let full_error = diagnostics
                        .into_iter()
                        .map(|d| {
                            Pretty::new(&d, &filename, &source).to_string()
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    anyhow::anyhow!("{}", full_error)
                })?;
                ctx.from_parsed(parsed)
            }
            // Assume order violations have been taken care of first.
            ViolationKind::MatchingFunctionMissing(fn_hir, index) => {
                let Some(contract_hir) = ctx.hir.find_contract() else {
                    return Ok(ctx);
                };
                let Some(contract_sol) = find_contract(&ctx.pt) else {
                    return Ok(ctx);
                };

                let offset = get_insertion_offset(
                    &contract_sol,
                    contract_hir,
                    *index,
                    &ctx.src,
                );
                ctx.insert_function_at(fn_hir, offset);

                let source = ctx.src.clone();
                let filename = ctx.sol.to_string_lossy();
                let parsed = parse(&source).map_err(|diagnostics| {
                    let full_error = diagnostics
                        .into_iter()
                        .map(|d| {
                            Pretty::new(&d, &filename, &source).to_string()
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    anyhow::anyhow!("{}", full_error)
                })?;
                ctx.from_parsed(parsed)
            }
            _ => ctx,
        })
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
    ctx.from_parsed(parsed)
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}: {}", "warn".yellow(), self.kind)?;
        if let Some(help_text) = self.kind.help() {
            writeln!(f, "     {} help: {}", "=".blue(), help_text)?;
        }
        if self.kind.is_fixable() {
            let file = self.location.file().replace(".t.sol", ".tree");
            write!(f, "     {} fix: run ", "+".blue())?;
            writeln!(f, "`bulloak check --fix {file}`")?;
        }
        writeln!(f, "   {} {}", "-->".blue(), self.location)?;

        Ok(())
    }
}

impl PartialEq for ViolationKind {
    fn eq(&self, other: &Self) -> bool {
        use ViolationKind::*;

        match (self, other) {
            (ContractMissing(a), ContractMissing(b)) => a == b,
            (
                ContractNameNotMatches(a1, a2),
                ContractNameNotMatches(b1, b2),
            ) => a1 == b1 && a2 == b2,
            (SolidityFileMissing(a), SolidityFileMissing(b)) => a == b,
            (FileUnreadable, FileUnreadable) => true,
            (
                FunctionOrderMismatch(f1, cur1, ins1),
                FunctionOrderMismatch(f2, cur2, ins2),
            ) =>
            // Compare on function name and the two positions.
            {
                f1.name == f2.name && cur1 == cur2 && ins1 == ins2
            }
            (
                MatchingFunctionMissing(f1, pos1),
                MatchingFunctionMissing(f2, pos2),
            ) =>
            // Compare on function identifier and the position.
            {
                f1.identifier == f2.identifier && pos1 == pos2
            }
            (ParsingFailed(e1), ParsingFailed(e2)) =>
            // Compare on the formatted error message.
            {
                e1.to_string() == e2.to_string()
            }

            // any mismatched variant
            _ => false,
        }
    }
}

/// Formats frontend errors into human-readable messages.
///
/// # Arguments
/// * `error` - Reference to an `anyhow::Error`
///
/// # Returns
/// A `String` containing the formatted error message
fn format_frontend_error(error: &anyhow::Error) -> String {
    if let Some(error) =
        error.downcast_ref::<bulloak_syntax::tokenizer::Error>()
    {
        format!("an error occurred while parsing the tree: {}", error.kind())
    } else if let Some(error) =
        error.downcast_ref::<bulloak_syntax::parser::Error>()
    {
        format!("an error occurred while parsing the tree: {}", error.kind())
    } else if let Some(error) =
        error.downcast_ref::<crate::hir::combiner::Error>()
    {
        format!("an error occurred while parsing the tree: {}", error.kind())
    } else if error.downcast_ref::<bulloak_syntax::semantics::Error>().is_some()
    {
        "at least one semantic error occurred while parsing the tree".to_owned()
    } else {
        "an error occurred while parsing the solidity file".to_owned()
    }
}
