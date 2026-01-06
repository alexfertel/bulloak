#![doc = include_str!("../README.md")]
use std::process;

mod backend;
mod check;
mod cli;
mod glob;
mod scaffold;

fn main() {
    if let Err(e) = crate::cli::run() {
        eprintln!("Error: {e:?}");
        process::exit(1);
    }
}
