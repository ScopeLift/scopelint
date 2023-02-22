#![doc = include_str!("../README.md")]
#![warn(missing_docs, unreachable_pub, unused, rust_2021_compatibility)]
#![warn(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]
#![allow(clippy::multiple_crate_versions)]
use clap::Parser;
use scopelint::config::Opts;
use std::process;

fn main() {
    let opts = Opts::parse();

    if let Err(_err) = scopelint::run(&opts) {
        // All warnings/errors have already been logged.
        process::exit(1);
    }
}
