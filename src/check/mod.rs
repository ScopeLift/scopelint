use colored::Colorize;
use once_cell::sync::Lazy;
use regex::Regex;
use solang_parser::pt::{
    ContractPart, FunctionDefinition, FunctionTy, SourceUnitPart, VariableAttribute,
    VariableDefinition,
};
use std::{error::Error, ffi::OsStr, fs, path::Path};
use walkdir::WalkDir;

pub mod checks;
pub mod report;
pub mod utils;

// A regex matching valid constant names, see the `validate_constant_names_regex` test for examples.
static RE_VALID_CONSTANT_NAME: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:[$_]*[A-Z0-9][$_]*){1,}$").unwrap());

/// Validates the code formatting, and print details on any conventions that are not being followed.
/// # Errors
/// TODO
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

impl Validate for VariableDefinition {
    fn validate(&self, content: &str, file: &Path) -> Vec<report::InvalidItem> {
        let mut invalid_items = Vec::new();
        let name = &self.name.name;

        // Validate constants and immutables are in ALL_CAPS.
        let is_constant = self
            .attrs
            .iter()
            .any(|a| matches!(a, VariableAttribute::Constant(_) | VariableAttribute::Immutable(_)));

        if is_constant && !is_valid_constant_name(name) {
            invalid_items.push(report::InvalidItem::new(
                report::Validator::Constant,
                file.display().to_string(),
                name.clone(),
                offset_to_line(content, self.loc.start()),
            ));
        }

        invalid_items
    }
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

            // Run checks.
            for element in pt.0 {
                match element {
                    SourceUnitPart::VariableDefinition(v) => {
                        results.add_items(v.validate(&content, dent.path()));
                    }
                    SourceUnitPart::ContractDefinition(c) => {
                        for el in c.parts {
                            if let ContractPart::VariableDefinition(v) = el {
                                results.add_items(v.validate(&content, dent.path()));
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    }
    Ok(results)
}

fn is_valid_constant_name(name: &str) -> bool {
    RE_VALID_CONSTANT_NAME.is_match(name)
}

// Converts the start offset of a `Loc` to `(line, col)`. Modified from https://github.com/foundry-rs/foundry/blob/45b9dccdc8584fb5fbf55eb190a880d4e3b0753f/fmt/src/helpers.rs#L54-L70
fn offset_to_line(content: &str, start: usize) -> usize {
    debug_assert!(content.len() > start);

    let mut line_counter = 1; // First line is `1`.
    for (offset, c) in content.chars().enumerate() {
        if c == '\n' {
            line_counter += 1;
        }
        if offset > start {
            return line_counter
        }
    }

    unreachable!("content.len() > start")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_constant_names_regex() {
        let allowed_names = vec![
            "MAX_UINT256",
            "256_MAXUINT",
            "256_MAX_11_UINT",
            "VARIABLE",
            "VARIABLE_NAME",
            "VARIABLE_NAME_",
            "VARIABLE___NAME",
            "VARIABLE_NAME_WOW",
            "VARIABLE_NAME_WOW_AS_MANY_UNDERSCORES_AS_YOU_WANT",
            "__VARIABLE",
            "_VARIABLE__NAME",
            "_VARIABLE_NAME__",
            "_VARIABLE_NAME_WOW",
            "_VARIABLE_NAME_WOW_AS_MANY_UNDERSCORES_AS_YOU_WANT",
            "$VARIABLE_NAME",
            "_$VARIABLE_NAME_",
            "$_VARIABLE_NAME$",
            "_$VARIABLE_NAME$_",
            "$_VARIABLE_NAME_$",
            "$_VARIABLE__NAME_",
        ];

        let disallowed_names = [
            "variable",
            "variableName",
            "_variable",
            "_variable_Name",
            "VARIABLe",
            "VARIABLE_name",
            "_VARIABLe",
            "_VARIABLE_name",
            "$VARIABLe",
            "$VARIABLE_name",
        ];

        for name in allowed_names {
            assert_eq!(is_valid_constant_name(name), true, "{name}");
        }

        for name in disallowed_names {
            assert_eq!(is_valid_constant_name(name), false, "{name}");
        }
    }
}
