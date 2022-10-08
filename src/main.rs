use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = scopelint::Config::build(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    if let Err(err) = scopelint::run(config) {
        eprintln!("\nExecution failed: {err}");
        process::exit(1);
    }
}
