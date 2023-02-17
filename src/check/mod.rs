use colored::Colorize;
use solang_parser::pt::{FunctionDefinition, FunctionTy};
use std::{error::Error, ffi::OsStr, fs, path::Path};
use walkdir::WalkDir;

pub mod checks;
pub mod report;
pub mod utils;

/// Validates the code formatting, and print details on any conventions that are not being followed.
pub fn run(taplo_opts: taplo::formatter::Options) -> Result<(), Box<dyn Error>> {
    let valid_names = validate_conventions();
    let valid_fmt = checks::formatting::run(taplo_opts);

    if valid_names.is_ok() && valid_fmt.is_ok() {
        Ok(())
    } else {
        Err("One or more checks failed, review above output".into())
    }
}

// =============================
// ======== Validations ========
// =============================

// -------- Top level validation methods --------

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

// -------- Validation implementation --------

trait Validate {
    fn validate(&self, content: &str, file: &Path) -> Vec<report::InvalidItem>;
}

trait Name {
    fn name(&self) -> String;
}

impl Name for FunctionDefinition {
    fn name(&self) -> String {
        match self.ty {
            FunctionTy::Constructor => "constructor".to_string(),
            FunctionTy::Fallback => "fallback".to_string(),
            FunctionTy::Receive => "receive".to_string(),
            FunctionTy::Function | FunctionTy::Modifier => self.name.as_ref().unwrap().name.clone(),
        }
    }
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
            let content = fs::read_to_string(dent.path())?;
            let (pt, _comments) = solang_parser::parse(&content, 0).expect("Parsing failed");

            results.add_items(checks::test_names::validate(dent.path(), &content, &pt)?);
            results.add_items(checks::src_names_internal::validate(dent.path(), &content, &pt)?);
            results.add_items(checks::script_one_pubic_run_method::validate(
                dent.path(),
                &content,
                &pt,
            )?);
            results.add_items(checks::constant_names::validate(dent.path(), &content, &pt)?);
        }
    }
    Ok(results)
}
