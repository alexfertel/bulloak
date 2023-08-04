use std::process;

use bulloak::{run, Config};
use clap::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::parse();

    if let Err(e) = run(&config) {
        println!("Application error: {e}");
        process::exit(1);
    }

    Ok(())
}
