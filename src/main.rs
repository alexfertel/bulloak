use std::process;

use bulloak::run;
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)] // Read from `Cargo.toml`
struct Config {
    tree: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::parse();

    if let Err(e) = run(&config.tree) {
        println!("Application error: {e}");
        process::exit(1);
    }

    Ok(())
}
