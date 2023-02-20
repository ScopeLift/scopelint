use colored::Colorize;
use std::{error::Error, ffi::OsStr, fs};
use walkdir::WalkDir;

pub mod report;
pub mod utils;
pub mod validators;

/// Validates the code formatting, and print details on any conventions that are not being followed.
pub fn run(taplo_opts: taplo::formatter::Options) -> Result<(), Box<dyn Error>> {
    // We run the formatting check separate to just indicate whether or not the user needs to format
    // the codebase, whereas the other validators return granular information about what to fix
    // since they currently can't be fixed automatically.
    let valid_names = validate_conventions();
    let valid_fmt = validators::formatting::validate(taplo_opts);

    if valid_names.is_ok() && valid_fmt.is_ok() {
        Ok(())
    } else {
        Err("One or more checks failed, review above output".into())
    }
}

// =============================
// ======== Validations ========
// =============================

fn validate_conventions() -> Result<(), Box<dyn Error>> {
    let paths = ["./src", "./script", "./test"];
    let results = validate(paths)?;

    if !results.is_valid() {
        eprint!("{results}");
        eprintln!("{}: Convention checks failed, see details above", "error".bold().red());
        return Err("Invalid names found".into())
    }
    Ok(())
}

// Core validation method that walks the directory and validates all Solidity files.
fn validate(paths: [&str; 3]) -> Result<report::Report, Box<dyn Error>> {
    let mut results = report::Report::default();

    for path in paths {
        for result in WalkDir::new(path) {
            let dent = match result {
                Ok(dent) => dent,
                Err(err) => {
                    eprintln!("{err}");
                    continue
                }
            };

            if !dent.file_type().is_file() || dent.path().extension() != Some(OsStr::new("sol")) {
                continue
            }

            // Get the parse tree (pt) of the file.
            let file = dent.path();
            let content = fs::read_to_string(file)?;
            let (pt, _comments) = solang_parser::parse(&content, 0).expect("Parsing failed");

            // Run all checks.
            results.add_items(validators::test_names::validate(file, &content, &pt)?);
            results.add_items(validators::src_names_internal::validate(file, &content, &pt)?);
            results
                .add_items(validators::script_one_pubic_run_method::validate(file, &content, &pt)?);
            results.add_items(validators::constant_names::validate(file, &content, &pt)?);
        }
    }
    Ok(results)
}
