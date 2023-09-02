use std::{io::Result, process};

fn main() -> Result<()> {
    if let Err(e) = bulloak::cli::run() {
        eprintln!("Error: {e:?}");
        process::exit(1);
    }

    Ok(())
}
