//! Defines the `bulloak scaffold` command.
//!
//! This command scaffolds a Solidity file from a spec `.tree` file.

use std::{
    fs,
    path::{Path, PathBuf},
};

use bulloak_foundry::constants::DEFAULT_SOL_VERSION;
use clap::Parser;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

use crate::{
    cli::{BackendKind, Cli},
    glob::expand_glob,
};

/// Generate test files based on your spec.
#[doc(hidden)]
#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
pub struct Scaffold {
    /// The set of tree files to generate from.
    ///
    /// Each test file will be named after its matching
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
    #[arg(short = 's', long, default_value = DEFAULT_SOL_VERSION)]
    pub solidity_version: String,
    /// Whether to add vm.skip(true) at the beginning of each test.
    #[arg(short = 'S', long = "vm-skip", default_value_t = false)]
    pub with_vm_skip: bool,
    /// Whether to emit modifiers.
    #[arg(short = 'm', long, default_value_t = false)]
    pub skip_modifiers: bool,
    /// Whether to capitalize and punctuate branch descriptions.
    #[arg(short = 'F', long = "format-descriptions", default_value_t = false)]
    pub format_descriptions: bool,
    /// The backend to use for code generation.
    #[arg(short = 'l', long = "lang", value_enum, default_value_t = BackendKind::Solidity)]
    pub backend_kind: BackendKind,
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
    pub(crate) fn run(&self, cfg: &Cli) {
        let mut files = Vec::with_capacity(self.files.len());
        for pattern in &self.files {
            match expand_glob(pattern.clone()) {
                Ok(iter) => files.extend(iter),
                Err(e) => {
                    eprintln!(
                        "{}: could not expand {}: {}",
                        "warn".yellow(),
                        pattern.display(),
                        e
                    );
                }
            }
        }

        let errors = files
            .iter()
            .filter_map(|file| {
                self.process_file(file, &cfg)
                    .map_err(|e| (file.as_path(), e))
                    .err()
            })
            .collect::<Vec<_>>();

        if !errors.is_empty() {
            Scaffold::report_errors(&errors);
            std::process::exit(1);
        }
    }

    /// Processes a single input file using dynamic dispatch.
    ///
    /// This method reads the input file, scaffolds the code using the backend,
    /// formats it, and either writes it to a file or prints to stdout.
    fn process_file(&self, file: &Path, cfg: &Cli) -> anyhow::Result<()> {
        let text = fs::read_to_string(file)?;
        let backend = self.backend_kind.get(cfg);

        let emitted = backend.scaffold(&text)?;

        let output_file = backend.test_filename(&file.to_path_buf())?;

        if self.write_files {
            self.write_file(&emitted, &output_file);
        } else {
            println!("{emitted}");
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
                "    {} The corresponding testfile already exists",
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
    fn report_errors(errors: &[(&Path, anyhow::Error)]) {
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
