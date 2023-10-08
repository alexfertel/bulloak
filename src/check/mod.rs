//! Defines the `bulloak check` command.
//!
//! This command performs checks on the relationship between a bulloak tree and a
//! Solidity file.

use std::path::PathBuf;

use clap::Parser;
use owo_colors::OwoColorize;
use violation::Violation;

use self::context::Context;
use self::rules::Checker;

mod context;
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
        let mut violations = Vec::new();
        for tree_path in self.files {
            let ctx = match Context::new(tree_path.clone()) {
                Ok(ctx) => ctx,
                Err(violation) => {
                    violations.push(violation);
                    continue;
                }
            };

            violations.append(&mut rules::structural_match::StructuralMatcher::check(&ctx));
        }

        exit(&violations);

        Ok(())
    }
}

fn exit(violations: &[Violation]) {
    if violations.is_empty() {
        println!(
            "{}",
            "All checks completed successfully! No issues found.".green()
        );
    } else {
        for violation in violations {
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
