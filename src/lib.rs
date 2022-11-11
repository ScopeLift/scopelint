#![doc = include_str!("../README.md")]
#![warn(missing_docs, unreachable_pub, unused, rust_2021_compatibility)]
#![warn(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]

use colored::Colorize;
use regex::Regex;
use solang_parser::pt::{
    ContractPart, FunctionAttribute, SourceUnitPart, VariableAttribute, Visibility,
};
use std::{error::Error, ffi::OsStr, fmt, fs, process};
use walkdir::WalkDir;

// ========================
// ======== Config ========
// ========================

// Using this enum and struct to simplify future changes if we allow more
// granularity, though this is probably overkill, especially since we'd likely
// use clap if input arguments get more complex.
enum Mode {
    Format,
    Check,
    Version,
}

/// Program configuration. Valid modes are `fmt` and `check`.
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

fn version() {
    println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}

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
    validate_names()
}

fn check(taplo_opts: taplo::formatter::Options) -> Result<(), Box<dyn Error>> {
    let valid_names = validate_names();
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

fn validate_names() -> Result<(), Box<dyn Error>> {
    let paths = ["./src", "./script", "./test"];
    let results = validate(paths)?;

    if !results.is_valid() {
        eprint!("{results}");
        eprintln!("{}: Naming conventions failed, see details above", "error".bold().red());
        return Err("Invalid names found".into())
    }
    Ok(())
}

enum Validator {
    Constant,
    Script,
    Src,
    Test,
}

struct InvalidItem {
    kind: Validator,
    file: String, // File name.
    text: String, // Incorrectly named item.
}

impl InvalidItem {
    fn description(&self) -> String {
        match self.kind {
            Validator::Test => {
                format!("Invalid test name in {}: {}", self.file, self.text)
            }
            Validator::Constant => {
                format!("Invalid constant or immutable name in {}: {}", self.file, self.text)
            }
            Validator::Script => format!("Invalid script interface in {}", self.file),
            Validator::Src => {
                format!("Invalid src method name in {}: {}", self.file, self.text)
            }
        }
    }
}

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
    const fn new() -> Self {
        Self { invalid_items: Vec::new() }
    }

    fn is_valid(&self) -> bool {
        self.invalid_items.is_empty()
    }
}

fn validate(paths: [&str; 3]) -> Result<ValidationResults, Box<dyn Error>> {
    let mut results = ValidationResults::new();

    for path in paths {
        let is_test = path == "./test";
        let is_src = path == "./src";
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
                    SourceUnitPart::ContractDefinition(c) => {
                        for el in c.parts {
                            match el {
                                ContractPart::VariableDefinition(v) => {
                                    let name = v.name.name;
                                    let is_constant = v.attrs.iter().any(|a| {
                                        matches!(
                                            a,
                                            VariableAttribute::Constant(_) |
                                                VariableAttribute::Immutable(_)
                                        )
                                    });
                                    if is_constant && !is_valid_constant_name(&name) {
                                        results.invalid_items.push(InvalidItem {
                                            kind: Validator::Constant,
                                            file: dent.path().display().to_string(),
                                            text: name,
                                        });
                                    }
                                }
                                ContractPart::FunctionDefinition(f) => {
                                    // Validate test function naming convention.
                                    let name = f.name.unwrap().name;
                                    if is_test && !is_valid_test_name(&name) {
                                        results.invalid_items.push(InvalidItem {
                                            kind: Validator::Test,
                                            file: dent.path().display().to_string(),
                                            text: name.clone(),
                                        });
                                    }

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

                                    if is_src && is_private && !is_valid_src_name(&name) {
                                        results.invalid_items.push(InvalidItem {
                                            kind: Validator::Src,
                                            file: dent.path().display().to_string(),
                                            text: name,
                                        });
                                    }
                                }
                                ContractPart::StructDefinition(_) |
                                ContractPart::EventDefinition(_) |
                                ContractPart::EnumDefinition(_) |
                                ContractPart::ErrorDefinition(_) |
                                ContractPart::TypeDefinition(_) |
                                ContractPart::StraySemicolon(_) |
                                ContractPart::Using(_) => (),
                            }
                        }
                    }
                    SourceUnitPart::PragmaDirective(_, _, _) |
                    SourceUnitPart::ImportDirective(_) |
                    SourceUnitPart::EnumDefinition(_) |
                    SourceUnitPart::StructDefinition(_) |
                    SourceUnitPart::EventDefinition(_) |
                    SourceUnitPart::ErrorDefinition(_) |
                    SourceUnitPart::FunctionDefinition(_) |
                    SourceUnitPart::VariableDefinition(_) |
                    SourceUnitPart::TypeDefinition(_) |
                    SourceUnitPart::Using(_) |
                    SourceUnitPart::StraySemicolon(_) => (),
                }
            }

            // Validate scripts only have a single run method.
            // TODO Script checks don't really fit nicely into InvalidItem, refactor needed to log
            // more details about the invalid script's ABI.
            if num_public_script_methods > 1 {
                results.invalid_items.push(InvalidItem {
                    kind: Validator::Script,
                    file: dent.path().display().to_string(),
                    text: String::new(),
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
    let regex = Regex::new(r"test(Fork)?(Fuzz)?_(Revert(If_|When_){1})?\w+").unwrap();
    regex.is_match(name)
}

fn is_valid_src_name(name: &str) -> bool {
    name.starts_with('_')
}

fn is_valid_constant_name(name: &str) -> bool {
    // Make sure it's ALL_CAPS: https://regex101.com/r/Pv9mD8/1
    let regex = Regex::new(r"^_?[A-Z]+(?:_{0,1}[A-Z]+)*$").unwrap();
    regex.is_match(name)
}
