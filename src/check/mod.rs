use std::{fs, path::PathBuf};

use clap::Parser;

use violation::Violation;
use violation::ViolationKind;

use self::rules::Checker;
use self::rules::Context;

mod rules;
pub(crate) mod violation;

/// The cli interface for the `bulloak check` command.
#[derive(Debug, Parser)]
pub struct Check {
    /// .tree files to process.
    files: Vec<PathBuf>,
}

impl Check {
    /// Entrypoint for `bulloak check`.
    ///
    /// Note that we don't deal with solang_parser errors at all.
    pub fn run(self: Check) -> anyhow::Result<()> {
        let mut violations: Vec<Violation> = Vec::new();

        for tree_path in self.files.iter() {
            // Get the path to the output file.
            let mut sol_path = tree_path.clone();
            sol_path.set_extension("t.sol");

            let tree_path_str = tree_path.to_string_lossy().into_owned();
            if !sol_path.exists() {
                violations.push(Violation::new(ViolationKind::FileMissing(
                    tree_path_str.clone(),
                )))
            }

            let tree = match try_read_to_string(&tree_path) {
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
                &solang_parser::parse(&code, 0).expect("should parse the solidity file");

            let sol_path_str = sol_path.to_string_lossy().into_owned();
            let ctx = Context {
                tree_hir,
                sol_ast,
                tree_path: &tree_path_str,
                sol_path: &sol_path_str,
            };
            violations.append(&mut rules::structural_match::StructuralMatcher::check(
                &ctx,
            )?);
        }

        if violations.len() > 0 {
            for violation in violations {
                eprintln!("{violation}");
            }
            std::process::exit(1);
        }

        Ok(())
    }
}

fn try_read_to_string(path: &PathBuf) -> Result<String, Violation> {
    fs::read_to_string(path).map_err(|_| {
        Violation::new(ViolationKind::FileUnreadable(
            path.to_string_lossy().into_owned(),
        ))
    })
}
