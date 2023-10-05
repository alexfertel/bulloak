//! Defines the `bulloak check` command.
//!
//! This command performs checks on the relationship between a bulloak tree and a
//! Solidity file.

use std::{fs, path::PathBuf};

use clap::Parser;

use owo_colors::OwoColorize;
use violation::Violation;
use violation::ViolationKind;

use self::location::Location;
use self::rules::Checker;
use self::rules::Context;

mod location;
mod rules;
mod utils;
pub(crate) mod violation;

/// Check that the tests match the spec.
#[derive(Debug, Parser)]
pub struct Check {
    /// The set of tree files to use as spec.
    ///
    /// Solidity file names are inferred from the specs.
    files: Vec<PathBuf>,
}

impl Check {
    /// Entrypoint for `bulloak check`.
    ///
    /// Note that we don't deal with `solang_parser` errors at all.
    pub fn run(self) -> anyhow::Result<()> {
        let mut violations: Vec<Violation> = Vec::new();

        for tree_path in &self.files {
            // Get the path to the output file.
            let mut sol_path = tree_path.clone();
            sol_path.set_extension("t.sol");

            let tree_path_str = tree_path.to_string_lossy().into_owned();
            if !sol_path.exists() {
                let filename = tree_path_str.clone();
                violations.push(Violation::new(
                    ViolationKind::FileMissing,
                    Location::File(filename),
                ));

                continue;
            }

            let tree = match try_read_to_string(tree_path) {
                Ok(code) => code,
                Err(violation) => {
                    violations.push(violation);
                    continue;
                }
            };
            let code = match try_read_to_string(&sol_path) {
                Ok(code) => code,
                Err(violation) => {
                    violations.push(violation);
                    continue;
                }
            };

            let tree_hir = &crate::hir::translate(&tree)?;
            let (sol_ast, _) =
                &solang_parser::parse(&code, 0).expect("should parse the Solidity file");

            let sol_path_str = sol_path.to_string_lossy().into_owned();
            let ctx = Context {
                tree_path: &tree_path_str,
                tree_hir,
                sol_path: &sol_path_str,
                sol_ast,
                sol_contents: &code,
            };
            violations.append(&mut rules::structural_match::StructuralMatcher::check(
                &ctx,
            )?);
        }

        exit(violations);

        Ok(())
    }
}

fn try_read_to_string(path: &PathBuf) -> Result<String, Violation> {
    fs::read_to_string(path).map_err(|_| {
        let path = path.to_string_lossy().into_owned();
        Violation::new(ViolationKind::FileUnreadable, Location::File(path))
    })
}

fn exit(violations: Vec<Violation>) {
    if violations.is_empty() {
        println!(
            "{}",
            "All checks completed successfully! No issues found.".green()
        );
    } else {
        for violation in &violations {
            eprint!("{violation}");
        }

        let pluralized_check = if violations.len() == 1 {
            "check"
        } else {
            "checks"
        };
        eprintln!(
            "\n{}: {} {} failed. See details above.",
            "error".bold().red(),
            violations.len(),
            pluralized_check
        );

        std::process::exit(1);
    }
}
