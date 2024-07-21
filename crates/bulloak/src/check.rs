//! Defines the `bulloak check` command.
//!
//! This command performs checks on the relationship between a bulloak tree and
//! a Solidity file.

use std::{fs, path::PathBuf};

use bulloak_foundry::{
    check::{
        context::Context,
        rules::{self, Checker},
        violation::fix_order,
    },
    sol::find_contract,
    violation::{Violation, ViolationKind},
};
use bulloak_syntax::utils::pluralize;
use clap::Parser;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

use crate::cli::Cli;

/// Check that the tests match the spec.
#[doc(hidden)]
#[derive(Debug, Parser, Clone, Serialize, Deserialize)]
pub struct Check {
    /// The set of tree files to use as spec.
    ///
    /// Solidity file names are inferred from the specs.
    pub files: Vec<PathBuf>,
    /// Whether to fix any issues found.
    #[arg(long, group = "fix-violations", default_value_t = false)]
    pub fix: bool,
    /// When `--fix` is passed, use `--stdout` to direct output
    /// to standard output instead of writing to files.
    #[arg(long, requires = "fix-violations", default_value_t = false)]
    pub stdout: bool,
    /// Whether to emit modifiers.
    #[arg(short = 'm', long, default_value_t = false)]
    pub skip_modifiers: bool,
}

impl Default for Check {
    fn default() -> Self {
        Check::parse_from(Vec::<String>::new())
    }
}

impl Check {
    /// Entrypoint for `bulloak check`.
    ///
    /// Note that we don't deal with `solang_parser` errors at all.
    pub(crate) fn run(&self, cfg: &Cli) -> anyhow::Result<()> {
        let mut violations = Vec::new();
        let ctxs: Vec<Context> = self
            .files
            .iter()
            .filter_map(|tree_path| {
                Context::new(tree_path.clone(), &cfg.into())
                    .map_err(|violation| violations.push(violation))
                    .ok()
            })
            .collect();

        if self.fix {
            let mut fixed_count = 0;
            for mut ctx in ctxs {
                let violations = rules::StructuralMatcher::check(&ctx);
                let fixable_count =
                    violations.iter().filter(|v| v.is_fixable()).count();

                // Process violations that affect function order first.
                let violations = violations.into_iter().filter(|v| {
                    !matches!(
                        v.kind,
                        ViolationKind::FunctionOrderMismatch(_, _, _)
                    )
                });
                for violation in violations {
                    ctx = violation.kind.fix(ctx);
                }

                // Second pass fixing order violations.
                let violations = rules::StructuralMatcher::check(&ctx);
                let violations: Vec<Violation> = violations
                    .into_iter()
                    .filter(|v| {
                        matches!(
                            v.kind,
                            ViolationKind::FunctionOrderMismatch(_, _, _)
                        )
                    })
                    .collect();
                if !violations.is_empty() {
                    if let Some(contract_sol) = find_contract(&ctx.pt) {
                        if let Some(contract_hir) =
                            ctx.hir.clone().find_contract()
                        {
                            ctx = fix_order(
                                &violations,
                                &contract_sol,
                                contract_hir,
                                ctx,
                            );
                        }
                    }
                }

                let sol = ctx.sol.clone();
                let formatted =
                    ctx.fmt().expect("should format the emitted solidity code");
                self.write(&formatted, sol);

                fixed_count += fixable_count;
            }

            let issue_literal = pluralize(fixed_count, "issue", "issues");
            println!(
                "\n{}: {} {} fixed.",
                "success".bold().green(),
                fixed_count,
                issue_literal
            );
        } else {
            for ctx in ctxs {
                violations.append(&mut rules::StructuralMatcher::check(&ctx));
            }

            exit(&violations);
        }

        Ok(())
    }

    /// Handles writing the output of the `check` command.
    ///
    /// If the `--stdout` flag was passed, then the output is printed to
    /// stdout, else it is written to the corresponding file.
    fn write(&self, output: &str, sol: PathBuf) {
        if self.stdout {
            println!("{} {}", "-->".blue(), sol.to_string_lossy());
            println!("{}", output.trim());
            println!("{}", "<--".blue());
        } else if let Err(e) = fs::write(sol, output) {
            eprintln!("{}: {e}", "warn".yellow());
        }
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
            eprintln!("{violation}");
        }

        let check_literal = pluralize(violations.len(), "check", "checks");
        eprint!(
            "{}: {} {} failed",
            "warn".bold().yellow(),
            violations.len(),
            check_literal
        );
        let fixable_count =
            violations.iter().filter(|v| v.is_fixable()).count();
        if fixable_count > 0 {
            let fix_literal = pluralize(fixable_count, "fix", "fixes");
            eprintln!(
                " (run `bulloak check --fix <.tree files>` to apply {fixable_count} {fix_literal})"
            );
        } else {
            eprintln!();
        }

        std::process::exit(1);
    }
}
