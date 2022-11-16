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
use std::{error::Error, ffi::OsStr, fmt, fs, process};
use walkdir::{DirEntry, WalkDir};

// A regex matching test names such as `test_AddsTwoNumbers` or
// `testFuzz_RevertIf_CallerIsUnauthorized`.
static RE_VALID_TEST_NAME: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"test(Fork)?(Fuzz)?_(Revert(If_|When_){1})?\w+").unwrap());

// A regex to ensure constant and immutable variables are in ALL_CAPS: https://regex101.com/r/Pv9mD8/1
static RE_VALID_CONSTANT_NAME: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^_?[A-Z]+(?:_{0,1}[A-Z]+)*$").unwrap());

// ========================
// ======== Config ========
// ========================

enum Mode {
    Format,
    Check,
    Version,
}

/// Program configuration. Valid modes are `fmt`, `check`, and `--version`.
pub struct Config {
    mode: Mode,
}

impl Config {
    /// Create a new configuration from the command line arguments.
    /// # Errors
    /// Errors if too many arguments are provided, or an invalid mode is provided.
    pub fn build(args: &[String]) -> Result<Self, &'static str> {
        match args.len() {
            1 => Ok(Self { mode: Mode::Format }), // Default to format if no args provided.
            2 => match args[1].as_str() {
                "fmt" => Ok(Self { mode: Mode::Format }),
                "check" => Ok(Self { mode: Mode::Check }),
                "--version" | "-v" => Ok(Self { mode: Mode::Version }),
                _ => Err("Unrecognized mode: Must be 'fmt', 'check', or '--version'"),
            },
            _ => Err("Too many arguments"),
        }
    }
}

// ===========================
// ======== Execution ========
// ===========================

/// Takes the provided `config` and runs the program.
/// # Errors
/// Errors if the provided mode fails to run.
pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    // Configure formatting options, https://taplo.tamasfe.dev/.
    let taplo_opts = taplo::formatter::Options {
        allowed_blank_lines: 1,
        indent_entries: true,
        reorder_keys: true,
        ..Default::default()
    };

    // Execute commands.
    match config.mode {
        Mode::Format => fmt(taplo_opts),
        Mode::Check => check(taplo_opts),
        Mode::Version => {
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

enum Validator {
    Constant,
    Script,
    Src,
    Test,
}

struct InvalidItem {
    // TODO Map solang `File` info to line number.
    kind: Validator,
    file: String, // File name.
    text: String, // Incorrectly named item.
    line: usize,  // Line number.
}

impl InvalidItem {
    fn description(&self) -> String {
        match self.kind {
            Validator::Test => {
                format!("Invalid test name in {} on line {}: {}", self.file, self.line, self.text)
            }
            Validator::Constant => {
                format!(
                    "Invalid constant or immutable name in {} on line {}: {}",
                    self.file, self.line, self.text
                )
            }
            Validator::Script => format!("Invalid script interface in {}", self.file),
            Validator::Src => {
                format!(
                    "Invalid src method name in {} on line {}: {}",
                    self.file, self.line, self.text
                )
            }
        }
    }
}

#[derive(Default)]
struct ValidationResults {
    invalid_items: Vec<InvalidItem>,
}

impl fmt::Display for ValidationResults {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        for item in &self.invalid_items {
            writeln!(f, "{}", item.description())?;
        }
        Ok(())
    }
}

impl ValidationResults {
    fn is_valid(&self) -> bool {
        self.invalid_items.is_empty()
    }
}

trait Validate {
    fn validate(&self, content: &str, dent: &DirEntry) -> Vec<InvalidItem>;
}

trait Name {
    fn name(&self) -> String;
}

impl Validate for VariableDefinition {
    fn validate(&self, content: &str, dent: &DirEntry) -> Vec<InvalidItem> {
        let mut invalid_items = Vec::new();
        let name = &self.name.name;

        // Validate constants and immutables are in ALL_CAPS.
        let is_constant = self
            .attrs
            .iter()
            .any(|a| matches!(a, VariableAttribute::Constant(_) | VariableAttribute::Immutable(_)));
        if is_constant && !is_valid_constant_name(name) {
            invalid_items.push(InvalidItem {
                kind: Validator::Constant,
                file: dent.path().display().to_string(),
                text: name.clone(),
                line: offset_to_line(content, self.loc.start()),
            });
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
    fn validate(&self, content: &str, dent: &DirEntry) -> Vec<InvalidItem> {
        let mut invalid_items = Vec::new();
        let name = &self.name();

        // Validate test names match the required pattern.
        if dent.path().starts_with("./test") && !is_valid_test_name(name) {
            invalid_items.push(InvalidItem {
                kind: Validator::Test,
                file: dent.path().display().to_string(),
                text: name.to_string(),
                line: offset_to_line(content, self.loc.start()),
            });
        }

        // Validate internal and private src methods start with an underscore.
        let is_private = self.attributes.iter().any(|a| match a {
            FunctionAttribute::Visibility(v) => {
                matches!(v, Visibility::Private(_) | Visibility::Internal(_))
            }
            _ => false,
        });

        if dent.path().starts_with("./src") && is_private && !name.starts_with('_') {
            invalid_items.push(InvalidItem {
                kind: Validator::Src,
                file: dent.path().display().to_string(),
                text: name.to_string(),
                line: offset_to_line(content, self.loc.start()),
            });
        }

        invalid_items
    }
}

// Core validation method that walks the filesystem and validates all Solidity files.
fn validate(paths: [&str; 3]) -> Result<ValidationResults, Box<dyn Error>> {
    let mut results = ValidationResults::default();

    for path in paths {
        let is_script = path == "./script";

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

            // Variables used to track status of checks that are file-wide.
            let mut num_public_script_methods = 0;

            // Run checks.
            for element in pt.0 {
                match element {
                    SourceUnitPart::FunctionDefinition(f) => {
                        results.invalid_items.extend(f.validate(&content, &dent));
                    }
                    SourceUnitPart::VariableDefinition(v) => {
                        results.invalid_items.extend(v.validate(&content, &dent));
                    }
                    SourceUnitPart::ContractDefinition(c) => {
                        for el in c.parts {
                            match el {
                                ContractPart::VariableDefinition(v) => {
                                    results.invalid_items.extend(v.validate(&content, &dent));
                                }
                                ContractPart::FunctionDefinition(f) => {
                                    results.invalid_items.extend(f.validate(&content, &dent));

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
                                    if is_script && !is_private && name != "setUp" {
                                        num_public_script_methods += 1;
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
            // TODO Script checks don't really fit nicely into InvalidItem, refactor needed to log
            // more details about the invalid script's ABI.
            if is_script && num_public_script_methods > 1 {
                results.invalid_items.push(InvalidItem {
                    kind: Validator::Script,
                    file: dent.path().display().to_string(),
                    text: String::new(),
                    line: 0, // This spans multiple lines, so we don't have a line number.
                });
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
