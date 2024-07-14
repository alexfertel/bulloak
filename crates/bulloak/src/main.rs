#![doc = include_str!("../README.md")]
use std::process;

mod check;
mod cli;
mod constants;
mod scaffold;

fn main() {
    if let Err(e) = crate::cli::run() {
        eprintln!("Error: {e:?}");
        process::exit(1);
    }
}
