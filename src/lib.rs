#![doc = include_str!("../README.md")]
#![warn(missing_docs, unreachable_pub, unused, rust_2021_compatibility)]
#![warn(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]

use colored::Colorize;
use once_cell::sync::Lazy;

use regex::Regex;
use solang_parser::pt::{
    ContractPart, FunctionAttribute, FunctionDefinition, FunctionTy, SourceUnitPart,
    VariableAttribute, VariableDefinition, Visibility,
};
use std::{error::Error, ffi::OsStr, fs, path::Path, process};
use walkdir::WalkDir;

/// Program configuration. Valid modes are `fmt`, `check`, and `--version`.
pub mod config;

/// Utilities for formatting and printing a report of results.
pub mod report;

// A regex matching valid test names, see the `validate_test_names_regex` test for examples.
static RE_VALID_TEST_NAME: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^test(Fork)?(Fuzz)?(_Revert(If|When|On))?_(\w+)*$").unwrap());

// A regex matching valid constant names, see the `validate_constant_names_regex` test for examples.
static RE_VALID_CONSTANT_NAME: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:[$_]*[A-Z0-9][$_]*){1,}$").unwrap());

// ===========================
// ======== Execution ========
// ===========================

/// Takes the provided `config` and runs the program.
/// # Errors
/// Errors if the provided mode fails to run.
pub fn run(config: &config::Config) -> Result<(), Box<dyn Error>> {
    // Configure formatting options, https://taplo.tamasfe.dev/.
    let taplo_opts = taplo::formatter::Options {
        allowed_blank_lines: 1,
        indent_entries: true,
        reorder_keys: true,
        ..Default::default()
    };

    // Execute commands.
    match config.mode {
        config::Mode::Format => fmt(taplo_opts),
        config::Mode::Check => check(taplo_opts),
        config::Mode::Version => {
            version();
            Ok(())
        }
    }
}

// Print the package version.
fn version() {
    println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}

// Format the code, and print details on any invalid items.
fn fmt(taplo_opts: taplo::formatter::Options) -> Result<(), Box<dyn Error>> {
    // Format Solidity with forge
    let forge_status = process::Command::new("forge").arg("fmt").output()?;

    // Print any warnings/errors from `forge fmt`.
    if !forge_status.stderr.is_empty() {
        print!("{}", String::from_utf8(forge_status.stderr)?);
    }

    // Format `foundry.toml` with taplo.
    let config_orig = fs::read_to_string("./foundry.toml")?;
    let config_fmt = taplo::formatter::format(&config_orig, taplo_opts);
    fs::write("./foundry.toml", config_fmt)?;

    // Check naming conventions.
    validate_conventions()
}

// Validate the code formatting, and print details on any invalid items.
fn check(taplo_opts: taplo::formatter::Options) -> Result<(), Box<dyn Error>> {
    let valid_names = validate_conventions();
    let valid_fmt = validate_fmt(taplo_opts);

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

fn validate_fmt(taplo_opts: taplo::formatter::Options) -> Result<(), Box<dyn Error>> {
    // Check Solidity with `forge fmt`
    let forge_status = process::Command::new("forge").arg("fmt").arg("--check").output()?;

    // Print any warnings/errors from `forge fmt`.
    let stderr = String::from_utf8(forge_status.stderr)?;
    let forge_ok = forge_status.status.success() && stderr.is_empty();
    print!("{stderr}"); // Prints nothing if stderr is empty.

    // Check TOML with `taplo fmt`
    let config_orig = fs::read_to_string("./foundry.toml")?;
    let config_fmt = taplo::formatter::format(&config_orig, taplo_opts);
    let taplo_ok = config_orig == config_fmt;

    if !forge_ok || !taplo_ok {
        eprintln!(
            "{}: Formatting validation failed, run `scopelint fmt` to fix",
            "error".bold().red()
        );
        return Err("Invalid fmt found".into())
    }
    Ok(())
}

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

impl Validate for FunctionDefinition {
    fn validate(&self, content: &str, file: &Path) -> Vec<report::InvalidItem> {
        let mut invalid_items = Vec::new();
        let name = &self.name();

        // Validate test names match the required pattern.
        if file.starts_with("./test") && !is_valid_test_name(name) {
            invalid_items.push(report::InvalidItem::new(
                report::Validator::Test,
                file.display().to_string(),
                name.to_string(),
                offset_to_line(content, self.loc.start()),
            ));
        }

        // Validate internal and private src methods start with an underscore.
        let is_private = self.attributes.iter().any(|a| match a {
            FunctionAttribute::Visibility(v) => {
                matches!(v, Visibility::Private(_) | Visibility::Internal(_))
            }
            _ => false,
        });

        if file.starts_with("./src") && is_private && !name.starts_with('_') {
            invalid_items.push(report::InvalidItem::new(
                report::Validator::Src,
                file.display().to_string(),
                name.to_string(),
                offset_to_line(content, self.loc.start()),
            ));
        }

        invalid_items
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

            // Executable script files are expected to end with `.s.sol`, whereas non-executable
            // helper contracts in the scripts dir just end with `.sol`.
            let is_script =
                path == "./script" && dent.path().to_str().expect("Bad path").ends_with(".s.sol");

            // Get the parse tree (pt) of the file.
            let content = fs::read_to_string(dent.path())?;
            let (pt, _comments) = solang_parser::parse(&content, 0).expect("Parsing failed");

            // Variables used to track status of checks that are file-wide.
            let mut public_methods: Vec<String> = Vec::new();

            // Run checks.
            for element in pt.0 {
                match element {
                    SourceUnitPart::FunctionDefinition(f) => {
                        results.add_items(f.validate(&content, dent.path()));
                    }
                    SourceUnitPart::VariableDefinition(v) => {
                        results.add_items(v.validate(&content, dent.path()));
                    }
                    SourceUnitPart::ContractDefinition(c) => {
                        for el in c.parts {
                            match el {
                                ContractPart::VariableDefinition(v) => {
                                    results.add_items(v.validate(&content, dent.path()));
                                }
                                ContractPart::FunctionDefinition(f) => {
                                    results.add_items(f.validate(&content, dent.path()));

                                    let name = f.name();
                                    let is_private = f.attributes.iter().any(|a| match a {
                                        FunctionAttribute::Visibility(v) => {
                                            matches!(
                                                v,
                                                Visibility::Private(_) | Visibility::Internal(_)
                                            )
                                        }
                                        _ => false,
                                    });

                                    if is_script &&
                                        !is_private &&
                                        name != "setUp" &&
                                        name != "constructor"
                                    {
                                        public_methods.push(name);
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                    _ => (),
                }
            }

            // Validate scripts only have a single public run method, or no public methods (i.e.
            // it's a helper contract not a script).
            if is_script {
                // If we have no public methods, the `run` method is missing.
                match public_methods.len() {
                    0 => {
                        results.add_item(report::InvalidItem::new(
                            report::Validator::Script,
                            dent.path().display().to_string(),
                            "No `run` method found".to_string(),
                            0, // This spans multiple lines, so we don't have a line number.
                        ));
                    }
                    1 => {
                        if public_methods[0] != "run" {
                            results.add_item(report::InvalidItem::new(
                                report::Validator::Script,
                                dent.path().display().to_string(),
                                "The only public method must be named `run`".to_string(),
                                0,
                            ));
                        }
                    }
                    _ => {
                        results.add_item(report::InvalidItem::new(
                            report::Validator::Script,
                            dent.path().display().to_string(),
                            format!("Scripts must have a single public method named `run` (excluding `setUp`), but the following methods were found: {public_methods:?}"),
                            0,
                        ));
                    }
                }
            }
        }
    }
    Ok(results)
}

fn is_valid_test_name(name: &str) -> bool {
    if !name.starts_with("test") {
        return true // Not a test function, so return.
    }
    RE_VALID_TEST_NAME.is_match(name)
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
    fn validate_test_names_regex() {
        let allowed_names = vec![
            "test_Description",
            "test_Increment",
            "testFuzz_Description",
            "testFork_Description",
            "testForkFuzz_Description",
            "testForkFuzz_Description_MoreInfo",
            "test_RevertIf_Condition",
            "test_RevertWhen_Condition",
            "test_RevertOn_Condition",
            "test_RevertOn_Condition_MoreInfo",
            "testFuzz_RevertIf_Condition",
            "testFuzz_RevertWhen_Condition",
            "testFuzz_RevertOn_Condition",
            "testFuzz_RevertOn_Condition_MoreInfo",
            "testForkFuzz_RevertIf_Condition",
            "testForkFuzz_RevertWhen_Condition",
            "testForkFuzz_RevertOn_Condition",
            "testForkFuzz_RevertOn_Condition_MoreInfo",
            "testForkFuzz_RevertOn_Condition_MoreInfo_Wow",
            "testForkFuzz_RevertOn_Condition_MoreInfo_Wow_As_Many_Underscores_As_You_Want",
        ];

        let disallowed_names = [
            "test",
            "testDescription",
            "testDescriptionMoreInfo",
            // TODO The below are tough to prevent without regex look-ahead support.
            // "test_RevertIfCondition",
            // "test_RevertWhenCondition",
            // "test_RevertOnCondition",
            // "testFuzz_RevertIfDescription",
            // "testFuzz_RevertWhenDescription",
            // "testFuzz_RevertOnDescription",
            // "testForkFuzz_RevertIfCondition",
            // "testForkFuzz_RevertWhenCondition",
            // "testForkFuzz_RevertOnCondition",
        ];

        for name in allowed_names {
            assert_eq!(is_valid_test_name(name), true, "{name}");
        }

        for name in disallowed_names {
            assert_eq!(is_valid_test_name(name), false, "{name}");
        }
    }

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
