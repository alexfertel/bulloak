//! Defines the `bulloak scaffold` command.
//!
//! This command scaffolds a Solidity file from a spec `.tree` file.

use std::{fs, path::PathBuf};

use clap::Parser;
use forge_fmt::fmt;
use owo_colors::OwoColorize;

use crate::constants::INTERNAL_DEFAULT_SOL_VERSION;
use crate::hir::translate_and_combine_trees;
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
    /// Whether to add vm.skip(true) at the begining of each test.
    #[arg(short = 'S', long = "vm-skip", default_value = "false")]
    with_vm_skip: bool,
}

impl Scaffold {
    pub fn run(self) -> anyhow::Result<()> {
        let scaffolder = Scaffolder::new(&self.solidity_version, self.with_vm_skip);

        // For each input file, compile it and print it or write it
        // to the filesystem.
        let mut errors = Vec::with_capacity(self.files.len());
        for file in &self.files {
            let text = fs::read_to_string(file)?;
            match scaffolder.scaffold(&text) {
                Ok(emitted) => {
                    let emitted = fmt(&emitted).unwrap_or_else(|err| {
                        eprintln!("{}: {}", "WARN".yellow(), err);
                        emitted
                    });

                    if !self.write_files {
                        println!("{emitted}");
                        continue;
                    }

                    let file = self.to_test_file(file);
                    self.write_file(&emitted, &file);
                }
                Err(err) => {
                    errors.push((file, err));
                }
            };
        }

        if !errors.is_empty() {
            let error_count = errors.len();
            for (file, err) in errors {
                eprintln!("{err}");
                eprintln!("file: {}", file.display());
            }

            eprintln!(
                "\n{}: Could not scaffold {} files. Check the output above or run {}, which might prove helpful.",
                "warn".yellow(),
                error_count.yellow(),
                "bulloak check".blue()
            );

            std::process::exit(1);
        }

        Ok(())
    }

    /// Gets the `t.sol` path equivalent of `file`.
    fn to_test_file(&self, file: &PathBuf) -> PathBuf {
        let mut file = file.clone();
        file.set_extension("t.sol");
        file
    }

    /// Writes the provided `text` to `file`.
    ///
    /// If the file doesn't exist it will create it. If it exists,
    /// and `--force-write` was not passed, it will skip writing to the file.
    fn write_file(&self, text: &str, file: &PathBuf) {
        // Don't overwrite files unless `--force-write` was passed.
        if file.exists() && !self.force_write {
            eprintln!(
                "{}: Skipped emitting {:?}",
                "warn".yellow(),
                file.as_path().blue()
            );
            eprintln!(
                "    {} The corresponding `.t.sol` file already exists",
                "=".blue()
            );
            return;
        }

        if let Err(err) = fs::write(file, text) {
            eprintln!("{}: {err}", "error".red());
        };
    }
}

/// The overarching struct that generates Solidity
/// code from a `.tree` file.
pub struct Scaffolder<'s> {
    /// Sets a Solidity version for the test contracts.
    solidity_version: &'s str,
    /// Whether to add vm.skip(true) at the begining of each test.
    with_vm_skip: bool,
}

impl<'s> Scaffolder<'s> {
    /// Creates a new scaffolder with the provided configuration.
    #[must_use]
    pub fn new(solidity_version: &'s str, with_vm_skip: bool) -> Self {
        Scaffolder {
            solidity_version,
            with_vm_skip,
        }
    }

    /// Generates Solidity code from a `.tree` file.
    pub fn scaffold(&self, text: &str) -> crate::error::Result<String> {
        let hir = translate_and_combine_trees(text, self.with_vm_skip)?;
        let pt = sol::Translator::new(self.solidity_version, self.with_vm_skip).translate(&hir);
        let source = sol::Formatter::new().emit(pt);
        let formatted = fmt(&source).expect("should format the emitted solidity code");

        Ok(formatted)
    }
}
