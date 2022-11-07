#![doc = include_str!("../README.md")]
#![warn(missing_docs, unreachable_pub, unused, rust_2021_compatibility)]
#![warn(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]
use colored::Colorize;
use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = scopelint::Config::build(&args).unwrap_or_else(|err| {
        eprintln!("{}: Argument parsing failed with '{err}'", "error".bold().red());
        process::exit(1);
    });

    if let Err(_err) = scopelint::run(&config) {
        // All warnings/errors have already been logged.
        process::exit(1);
    }
}
