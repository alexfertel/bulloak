//! Defines the `bulloak scaffold` command.
//!
//! This command scaffolds a Solidity file from a spec `.tree` file.

use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use forge_fmt::fmt;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

use crate::{
    config::Config, constants::INTERNAL_DEFAULT_SOL_VERSION,
    hir::translate_and_combine_trees, sol,
};

pub mod emitter;
pub mod modifiers;

/// Generate Solidity tests based on your spec.
#[doc(hidden)]
#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
pub struct Scaffold {
    /// The set of tree files to generate from.
    ///
    /// Each Solidity file will be named after its matching
    /// tree spec.
    pub files: Vec<PathBuf>,
    /// Whether to write to files instead of stdout.
    ///
    /// This will write the output for each input file to the file
    /// specified at the root of the input file if the output file
    /// doesn't already exist. To overwrite, use `--force-write`
    /// together with `--write-files`.
    #[arg(short = 'w', long, group = "file-handling", default_value_t = false)]
    pub write_files: bool,
    /// When `--write-files` is passed, use `--force-write` to
    /// overwrite the output files.
    #[arg(
        short = 'f',
        long,
        requires = "file-handling",
        default_value_t = false
    )]
    pub force_write: bool,
    /// Sets a Solidity version for the test contracts.
    #[arg(short = 's', long, default_value = INTERNAL_DEFAULT_SOL_VERSION)]
    pub solidity_version: String,
    /// Whether to add vm.skip(true) at the begining of each test.
    #[arg(short = 'S', long = "vm-skip", default_value_t = false)]
    pub with_vm_skip: bool,
    /// Whether to emit modifiers.
    #[arg(short = 'm', long, default_value_t = false)]
    pub skip_modifiers: bool,
}

impl Default for Scaffold {
    fn default() -> Self {
        Scaffold::parse_from(Vec::<String>::new())
    }
}

impl Scaffold {
    /// Runs the scaffold command, processing all specified files.
    ///
    /// This method iterates through all input files, processes them, and either
    /// writes the output to files or prints to stdout based on the config.
    ///
    /// If any errors occur during processing, they are collected and reported.
    pub(crate) fn run(&self, cfg: &Config) -> anyhow::Result<()> {
        let errors: Vec<_> = self
            .files
            .iter()
            .filter_map(|file| {
                self.process_file(file, cfg)
                    .map_err(|e| (file.as_path(), e))
                    .err()
            })
            .collect();

        if !errors.is_empty() {
            self.report_errors(&errors);
            std::process::exit(1);
        }

        Ok(())
    }

    /// Processes a single input file.
    ///
    /// This method reads the input file, scaffolds the Solidity code, formats
    /// it, and either writes it to a file or prints it to stdout.
    fn process_file(&self, file: &Path, cfg: &Config) -> anyhow::Result<()> {
        let text = fs::read_to_string(file)?;
        let emitted = scaffold(&text, cfg)?;
        let formatted = fmt(&emitted).unwrap_or_else(|err| {
            eprintln!("{}: {}", "WARN".yellow(), err);
            emitted
        });

        if self.write_files {
            let file = file.with_extension("t.sol");
            self.write_file(&formatted, &file);
        } else {
            println!("{formatted}");
        }

        Ok(())
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

    /// Reports errors that occurred during file processing.
    ///
    /// This method prints error messages for each file that failed to process,
    /// along with a summary of the total number of failed files.
    fn report_errors(&self, errors: &[(&Path, anyhow::Error)]) {
        for (file, err) in errors {
            eprintln!("{err}");
            eprintln!("file: {}", file.display());
        }

        eprintln!(
            "\n{}: Could not scaffold {} files. Check the output above or run {}, which might prove helpful.",
            "warn".yellow(),
            errors.len().yellow(),
            "bulloak check".blue()
        );
    }
}

/// Generates Solidity code from a `.tree` file.
///
/// This function takes the content of a `.tree` file and a configuration,
/// translates it to an intermediate representation, then to Solidity, and
/// finally formats the resulting Solidity code.
pub fn scaffold(text: &str, cfg: &Config) -> crate::error::Result<String> {
    let hir = translate_and_combine_trees(text, cfg)?;
    let pt = sol::Translator::new(cfg).translate(&hir);
    let source = sol::Formatter::new().emit(pt);
    let formatted =
        fmt(&source).expect("should format the emitted solidity code");

    Ok(formatted)
}
