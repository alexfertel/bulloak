//! Defines the `bulloak scaffold` command.
//!
//! This command scaffolds a Solidity file from a spec `.tree` file.

use std::{fs, path::PathBuf};

use clap::Parser;
use forge_fmt::fmt;
use owo_colors::OwoColorize;

use crate::constants::INTERNAL_DEFAULT_SOL_VERSION;
use crate::utils::translate_and_combine_trees;
use crate::sol;

pub mod emitter;
pub mod modifiers;

/// Generate Solidity tests based on your spec.
#[doc(hidden)]
#[derive(Parser, Debug)]
pub struct Scaffold {
    /// The set of tree files to generate from.
    ///
    /// Each Solidity file will be named after its matching
    /// tree spec.
    files: Vec<PathBuf>,
    /// Whether to write to files instead of stdout.
    ///
    /// This will write the output for each input file to the file
    /// specified at the root of the input file if the output file
    /// doesn't already exist. To overwrite, use `--force-write`
    /// together with `--write-files`.
    #[arg(short = 'w', long, group = "file-handling", default_value = "false")]
    write_files: bool,
    /// When `--write-files` is passed, use `--force-write` to
    /// overwrite the output files.
    #[arg(short = 'f', long, requires = "file-handling", default_value = "false")]
    force_write: bool,
    /// Sets a Solidity version for the test contracts.
    #[arg(short = 's', long, default_value = INTERNAL_DEFAULT_SOL_VERSION)]
    solidity_version: String,
}

impl Scaffold {
    pub fn run(self) -> anyhow::Result<()> {
        let scaffolder = Scaffolder::new(&self.solidity_version);

        // For each input file, compile it and print it or write it
        // to the filesystem.
        for file in &self.files {
            let text = fs::read_to_string(file)?;
            match scaffolder.scaffold(&text) {
                Ok(emitted) => {
                    let emitted = fmt(&emitted).unwrap_or_else(|e| {
                        eprintln!("{}: {}", "WARN".yellow(), e);
                        emitted
                    });

                    if self.write_files {
                        let mut output_path = file.clone();

                        // Get the path to the output file.
                        output_path.set_extension("t.sol");

                        // Don't overwrite files unless `--force-write` was passed.
                        if output_path.exists() && !self.force_write {
                            eprintln!(
                                "{}: Skipped emitting {:?}",
                                "warn".yellow(),
                                file.as_path().blue()
                            );
                            eprintln!(
                                "    {} The corresponding `.t.sol` file already exists",
                                "=".blue()
                            );
                            continue;
                        }

                        if let Err(e) = fs::write(output_path, emitted) {
                            eprintln!("{}: {e}", "error".red());
                        };
                    } else {
                        println!("{emitted}");
                    }
                }
                Err(err) => {
                    eprintln!("{err}");
                    eprintln!("file: {}", file.display());
                    std::process::exit(1);
                }
            };
        }

        Ok(())
    }
}

/// The overarching struct that generates Solidity
/// code from a `.tree` file.
pub struct Scaffolder<'s> {
    /// Sets a Solidity version for the test contracts.
    solidity_version: &'s str,
}

impl<'s> Scaffolder<'s> {
    /// Creates a new scaffolder with the provided configuration.
    #[must_use]
    pub fn new(solidity_version: &'s str) -> Self {
        Scaffolder { solidity_version }
    }

    /// Generates Solidity code from a `.tree` file.
    pub fn scaffold(&self, text: &str) -> crate::error::Result<String> {
        let hir = translate_and_combine_trees(text)?;
        let pt = sol::Translator::new(self.solidity_version).translate(&hir);
        let source = sol::Formatter::new().emit(pt);
        let formatted = fmt(&source).expect("should format the emitted solidity code");

        Ok(formatted)
    }
}
