use std::{fs, path::PathBuf};

use clap::Parser;

use crate::span::{Position, Span};
use violation::Violation;
use violation::ViolationKind;

mod rules;
pub(crate) mod violation;

/// The cli interface for the `bulloak check` command.
#[derive(Debug, Parser)]
pub struct Check {
    /// .tree files to process.
    files: Vec<PathBuf>,
}

impl Check {
    pub fn run(self: Check) -> anyhow::Result<()> {
        let mut violations: Vec<Violation> = Vec::new();

        for tree_path in self.files.iter() {
            let mut sol_path = tree_path.clone();

            // Get the path to the output file.
            sol_path.set_extension("t.sol");

            if !sol_path.exists() {
                violations.push(Violation::new(
                    ViolationKind::FileMissing(tree_path.to_string_lossy().into_owned()),
                    Span::splat(Position::new(0, 1, 1)),
                    "".to_string(),
                ))
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

            let tree_ast = crate::syntax::parse(&tree)?;
            let (sol_ast, _) =
                solang_parser::parse(&code, 0).expect("should parse the solidity file");
        }

        for violation in violations {
            eprintln!("{violation}");
        }

        Ok(())
    }
}

fn try_read_to_string(path: &PathBuf) -> Result<String, Violation> {
    fs::read_to_string(path).map_err(|_| {
        Violation::new(
            ViolationKind::FileUnreadable(path.to_string_lossy().into_owned()),
            Span::splat(Position::new(0, 1, 1)),
            "".to_string(),
        )
    })
}
