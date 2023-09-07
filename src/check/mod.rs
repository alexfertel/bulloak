use std::io::Result;

use clap::Parser;

/// The cli interface for the `bulloak check` command.
#[derive(Debug, Parser)]
pub struct Check {}

impl Check {
    pub fn run(self: Check) -> Result<()> {
        Ok(())
    }
}
