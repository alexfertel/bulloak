use std::fs;
use std::io::Result;
use std::path::PathBuf;
use std::process;

use bulloak::*;
use clap::Parser;

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
    /// specified at the root of the input file.
    #[arg(short = 'w', long = "write-files")]
    write_files: bool,
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
    for file in config.files.iter() {
        let text = fs::read_to_string(file)?;
        match scaffold(&text, config.with_actions_as_comments, config.indent) {
            Ok(compiled) => {
                if config.write_files {
                    let mut path = file.clone();
                    path.set_file_name(compiled.output_file);
                    fs::write(path, compiled.emitted)?;
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
