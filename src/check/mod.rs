use std::{fs, io::Result, path::PathBuf};

use clap::Parser;

/// The cli interface for the `bulloak check` command.
#[derive(Debug, Parser)]
pub struct Check {
    /// .tree files to process.
    files: Vec<PathBuf>,
}

impl Check {
    pub fn run(self: Check) -> Result<()> {
        for tree_path in self.files.iter() {
            let mut sol_path = tree_path.clone();

            // Get the path to the output file.
            sol_path.set_extension("t.sol");

            let _tree = fs::read_to_string(tree_path)?;
            let code = fs::read_to_string(sol_path)?;

            let (pt, _) = solang_parser::parse(&code, 0).expect("should parse the solidity file");

            println!("{pt:?}");
        }

        Ok(())
    }
}
