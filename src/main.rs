use std::fs;
use std::io::Result;
use std::path::PathBuf;
use std::process;

use bulloak::*;
use clap::Parser;
use owo_colors::OwoColorize;

#[derive(Parser)]
#[command(author, version, about, long_about = None)] // Read from `Cargo.toml`
struct Config {
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
}

fn main() -> Result<()> {
    let config = Config::parse();

    if let Err(e) = run(&config) {
        println!("Application error: {e}");
        process::exit(1);
    }

    Ok(())
}

fn run(config: &Config) -> Result<()> {
    // For each input file, compile it and print it or write it
    // to the filesystem.
    for file in config.files.iter() {
        let text = fs::read_to_string(file)?;
        match scaffold(&text, config.with_actions_as_comments, config.indent) {
            Ok(compiled) => {
                if config.write_files {
                    let mut output_path = file.clone();

                    // Get the path to the output file.
                    output_path.set_file_name(compiled.output_file);

                    // Don't overwrite files unless `--force-write` was passed.
                    if output_path.try_exists().is_ok() && !config.force_write {
                        eprintln!(
                            "{}: Skipped emitting to {:?}.\n      The file {:?} already exists.",
                            "WARN".yellow(),
                            file.as_path().blue(),
                            output_path.as_path().blue()
                        );
                        continue;
                    }

                    if let Err(e) = fs::write(output_path, compiled.emitted) {
                        eprintln!("{}: {}", "ERROR".red(), e);
                    };
                } else {
                    println!("{}", compiled.emitted);
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
