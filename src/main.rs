#![doc = include_str!("../README.md")]
use std::process;

fn main() {
    if let Err(e) = bulloak::config::run() {
        eprintln!("Error: {e:?}");
        process::exit(1);
    }
}
