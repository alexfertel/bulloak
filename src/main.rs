use std::{io::Result, process};

mod check;
mod cli;
mod scaffold;

fn main() -> Result<()> {
    if let Err(e) = cli::run() {
        eprintln!("Error: {e:?}");
        process::exit(1);
    }

    Ok(())
}
