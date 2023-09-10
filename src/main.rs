#![doc = include_str!("../README.md")]
use std::process;

use bulloak;

fn main() {
    if let Err(e) = bulloak::cli::run() {
        eprintln!("Error: {e:?}");
        process::exit(1);
    }
}
