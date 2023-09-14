use std::{fs, path::PathBuf};

use clap::Parser;
use owo_colors::OwoColorize;

use crate::{hir::translator, syntax};

pub mod emitter;
pub mod modifiers;

/// The cli interface for the `bulloak scaffold` command.
#[doc(hidden)]
#[derive(Parser, Debug)]
pub struct Scaffold {
    /// .tree files to process.
    files: Vec<PathBuf>,

    /// Whether to print `it` branches as comments
    /// in the output code.
    #[arg(short = 'c', default_value = "true")]
    with_actions_as_comments: bool,

    /// The indentation of the output code.
    #[arg(short = 'i', default_value = "2")]
    indent: usize,

    /// Whether to write to files instead of stdout.
    ///
    /// This will write the output for each input file to the file
    /// specified at the root of the input file if the output file
    /// doesn't already exist. To overwrite, use `--force-write`
    /// together with `--write-files`.
    #[arg(short = 'w', long, group = "file-handling")]
    write_files: bool,

    /// When `write_files` is specified, use `--force-write` to
    /// overwrite the output files.
    #[arg(short = 'f', long, requires = "file-handling", default_value = "false")]
    force_write: bool,

    /// Sets a solidity version for the test contracts.
    #[arg(short = 's', long, default_value = "0.8.0")]
    solidity_version: String,
}

impl Scaffold {
    pub fn run(self: Scaffold) -> anyhow::Result<()> {
        let scaffolder = Scaffolder::new(
            self.with_actions_as_comments,
            self.indent,
            &self.solidity_version,
        );
        // For each input file, compile it and print it or write it
        // to the filesystem.
        for file in self.files.iter() {
            let text = fs::read_to_string(file)?;
            match scaffolder.scaffold(&text) {
                Ok(emitted) => {
                    if self.write_files {
                        let mut output_path = file.clone();

                        // Get the path to the output file.
                        output_path.set_extension("t.sol");

                        // Don't overwrite files unless `--force-write` was passed.
                        if output_path.exists() && !self.force_write {
                            eprintln!(
                                "{}: Skipped emitting to {:?}.",
                                "WARN".yellow(),
                                file.as_path().bright_blue()
                            );
                            eprintln!(
                                "      The file {:?} already exists.",
                                output_path.as_path().bright_blue()
                            );
                            continue;
                        }

                        if let Err(e) = fs::write(output_path, emitted) {
                            eprintln!("{}: {}", "ERROR".red(), e);
                        };
                    } else {
                        println!("{}", emitted);
                    }
                }
                Err(err) => {
                    eprintln!("{}", err);
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
    /// Whether to print `it` branches as comments
    /// in the output code.
    with_comments: bool,
    /// The indentation of the output code.
    indent: usize,
    /// Sets a solidity version for the test contracts.
    solidity_version: &'s str,
}

impl<'s> Scaffolder<'s> {
    /// Creates a new scaffolder with the provided configuration.
    pub fn new(with_comments: bool, indent: usize, solidity_version: &'s str) -> Self {
        Scaffolder {
            with_comments,
            indent,
            solidity_version,
        }
    }

    /// Generates Solidity code from a `.tree` file.
    pub fn scaffold(&self, text: &str) -> crate::error::Result<String> {
        let ast = syntax::parse(text)?;
        let mut discoverer = modifiers::ModifierDiscoverer::new();
        let modifiers = discoverer.discover(&ast);
        let hir = translator::Translator::new(self.solidity_version).translate(&ast, modifiers);
        let emitted = emitter::Emitter::new(self.with_comments, self.indent).emit(&hir);

        Ok(emitted)
    }
}
