#![doc = include_str!("../README.md")]
#![warn(missing_docs, unreachable_pub, unused, rust_2021_compatibility)]
#![warn(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]

use grep::{
    matcher::Matcher,
    regex::RegexMatcher,
    searcher::{sinks::UTF8, BinaryDetection, SearcherBuilder},
};
use regex::Regex;
use std::{error::Error, fmt, fs, process};
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
                "version" => Ok(Self { mode: Mode::Version }),
                _ => Err("Unrecognized mode"),
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
    process::Command::new("forge").arg("fmt").output().expect("forge fmt failed");

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
    let forge_ok = forge_status.status.success();

    // Check TOML with `taplo fmt`
    let config_orig = fs::read_to_string("./foundry.toml")?;
    let config_fmt = taplo::formatter::format(&config_orig, taplo_opts);
    let taplo_ok = config_orig == config_fmt;

    if !forge_ok || !taplo_ok {
        eprintln!("Error: Formatting failed, run `scopelint fmt` to fix");
        return Err("Invalid fmt found".into())
    }
    Ok(())
}

fn validate_names() -> Result<(), Box<dyn Error>> {
    let paths = ["./src", "./script", "./test"];
    let results = validate(paths)?;

    if !results.is_valid() {
        eprintln!("{results}");
        eprintln!("Error: Naming conventions failed, see details above");
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
    line: u64,    // Line number.
    text: String, // Incorrectly named item.
}

impl InvalidItem {
    fn description(&self) -> String {
        match self.kind {
            Validator::Test => {
                format!("Invalid test name in {} on line {}: {}\n", self.file, self.line, self.text)
            }
            Validator::Constant => {
                format!(
                    "Invalid constant or immutable name in {} on line {}: {}\n",
                    self.file, self.line, self.text
                )
            }
            Validator::Script => format!("Invalid script interface in {}\n", self.file),
            Validator::Src => {
                format!(
                    "Invalid src method name in {} on line {}: {}\n",
                    self.file, self.line, self.text
                )
            }
        }
    }
}

struct ValidationResults {
    invalid_tests: Vec<InvalidItem>,
    invalid_constants: Vec<InvalidItem>,
    invalid_scripts: Vec<InvalidItem>,
    invalid_src: Vec<InvalidItem>,
}

impl fmt::Display for ValidationResults {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        for item in &self.invalid_tests {
            write!(f, "{}", item.description())?;
        }
        for item in &self.invalid_constants {
            write!(f, "{}", item.description())?;
        }
        for item in &self.invalid_scripts {
            write!(f, "{}", item.description())?;
        }
        for item in &self.invalid_src {
            write!(f, "{}", item.description())?;
        }
        Ok(())
    }
}

impl ValidationResults {
    const fn new() -> Self {
        Self {
            invalid_tests: Vec::new(),
            invalid_constants: Vec::new(),
            invalid_scripts: Vec::new(),
            invalid_src: Vec::new(),
        }
    }

    fn is_valid(&self) -> bool {
        self.invalid_tests.is_empty() &&
            self.invalid_constants.is_empty() &&
            self.invalid_scripts.is_empty()
    }
}

fn validate(paths: [&str; 3]) -> Result<ValidationResults, Box<dyn Error>> {
    // Test and constant matchers are a single line, so we use `new_line_matcher`, but function
    // signatures may be multi-line, so we use `new`.
    let test_matcher = RegexMatcher::new_line_matcher(r"function\s*test\w+\(")?;
    let constant_matcher = RegexMatcher::new_line_matcher(r"\sconstant\s")?;
    let fn_matcher = RegexMatcher::new(r"function\s+\w+\([\w\s,]*\)[\w\s]*\{")?;

    let mut searcher = SearcherBuilder::new()
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .line_number(true)
        .build();

    let mut multiline_searcher = SearcherBuilder::new()
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .line_number(true)
        .multi_line(true)
        .build();

    let mut results = ValidationResults::new();

    for path in paths {
        for result in WalkDir::new(path) {
            let dent = match result {
                Ok(dent) => dent,
                Err(err) => {
                    eprintln!("{err}");
                    continue
                }
            };

            if !dent.file_type().is_file() {
                continue
            }

            // Validate test naming convention.
            searcher.search_path(
                &test_matcher,
                dent.path(),
                UTF8(|lnum, line| {
                    if let Some(i) = check_test(&test_matcher, &dent, lnum, line) {
                        results.invalid_tests.push(i);
                    }
                    Ok(true)
                }),
            )?;

            // Validate constant/immutable naming convention.
            searcher.search_path(
                &constant_matcher,
                dent.path(),
                UTF8(|lnum, line| {
                    if let Some(item) = check_constant(&dent, lnum, line) {
                        results.invalid_constants.push(item);
                    }
                    Ok(true)
                }),
            )?;

            // Validate src contract function names have leading underscores if internal/private.
            if path == "./src" {
                multiline_searcher.search_path(
                    &fn_matcher,
                    dent.path(),
                    UTF8(|lnum, line| {
                        if let Some(item) = check_src_fn(&fn_matcher, &dent, lnum, line) {
                            results.invalid_src.push(item);
                        }
                        Ok(true)
                    }),
                )?;
            }

            // Validate scripts only have a single run method.
            if path == "./script" {
                if let Some(i) = check_script(&dent)? {
                    results.invalid_scripts.push(i);
                }
            }
        }
    }
    Ok(results)
}

fn check_test(
    matcher: &RegexMatcher,
    dent: &walkdir::DirEntry,
    lnum: u64,
    line: &str,
) -> Option<InvalidItem> {
    // We are guaranteed to find a match, so the unwrap is ok.
    let the_match = matcher.find(line.as_bytes()).unwrap().unwrap();
    let text = line[the_match].to_string();

    // Check if it matches our pattern.
    let pattern = r"test(Fork)?(Fuzz)?_(Revert(If_|When_){1})?\w+\(";
    let validator = RegexMatcher::new_line_matcher(pattern).unwrap();

    // If match is found, test name is good, otherwise it's bad.
    let match_result = validator.find(text.as_bytes()).unwrap();
    if match_result.is_some() {
        return None
    }

    let trimmed_test = text.trim();
    let item = InvalidItem {
        kind: Validator::Test,
        file: dent.path().to_str().unwrap().to_string(),
        line: lnum,
        // Trim off the leading "function " and remove the trailing "(".
        text: trimmed_test[9..trimmed_test.len() - 1].to_string(),
    };

    Some(item)
}

fn check_src_fn(
    matcher: &RegexMatcher,
    dent: &walkdir::DirEntry,
    lnum: u64,
    line: &str,
) -> Option<InvalidItem> {
    // We are guaranteed to find a match, so the unwrap is ok.
    let the_match = matcher.find(line.as_bytes()).unwrap().unwrap();
    let text = line[the_match].to_string();

    // Ensure public/external functions have no leading underscore, and internal/private functions
    // have a leading underscore.
    let vis_validator = RegexMatcher::new_line_matcher(r"\b(public|external)\b").unwrap();
    let is_public = vis_validator.find(text.as_bytes()).unwrap();
    let name = text[9..text.len() - 1].to_string();
    let first_char = name.trim().chars().next()?;

    if is_public.is_some() && first_char != '_' || is_public.is_none() && first_char == '_' {
        return None
    }

    let item = InvalidItem {
        kind: Validator::Src,
        file: dent.path().to_str().unwrap().to_string(),
        line: lnum,
        // Trim off the leading "function " and remove the trailing "(".
        text: name,
    };
    Some(item)
}

fn check_constant(dent: &walkdir::DirEntry, lnum: u64, line: &str) -> Option<InvalidItem> {
    // Found a constant/immutable, get the var name.
    let r = Regex::new(r"(;|=)").unwrap();
    let mut split_str = r.split(line);
    let var = split_str.next().expect("no match 1").split_whitespace().last().expect("no match 2");

    // Make sure it's ALL_CAPS: https://regex101.com/r/Pv9mD8/1
    let name_validator = RegexMatcher::new_line_matcher(r"^[A-Z]+(?:_{0,1}[A-Z]+)*$").unwrap();

    // If match is found, test name is good, otherwise it's bad.
    let match_result = name_validator.find(var.as_bytes()).unwrap();
    if match_result.is_some() {
        return None
    }

    let item = InvalidItem {
        kind: Validator::Constant,
        file: dent.path().to_str().unwrap().to_string(),
        line: lnum,
        text: var.to_string(),
    };
    Some(item)
}

fn check_script(dent: &walkdir::DirEntry) -> Result<Option<InvalidItem>, Box<dyn Error>> {
    let mut fns_found = 0;
    let mut found_run_fn = false;

    let text = fs::read_to_string(dent.path())?;
    let fn_regex = Regex::new(r"function\s*\w*\([\w\s,]*\)[\w\s]*\{").unwrap();
    let setup_regex = Regex::new(r"function\s*setUp\(").unwrap();
    let public_regex = Regex::new(r"\b(public|external)\b").unwrap();
    let run_regex = Regex::new(r"\brun\b").unwrap();

    for cap in fn_regex.captures_iter(&text) {
        let text = &cap[0];
        if !setup_regex.is_match(text) && public_regex.is_match(text) {
            fns_found += 1;
            found_run_fn = found_run_fn || run_regex.is_match(text);
        }
    }

    if fns_found == 1 && found_run_fn {
        Ok(None)
    } else {
        // We only return 1 item to summarize the file.
        // TODO Script checks don't really fit nicely into InvalidItem, refactor needed to log more
        // details about the invalid script's ABI.
        Ok(Some(InvalidItem {
            kind: Validator::Script,
            file: dent.path().to_str().unwrap().to_string(),
            line: u64::MAX,      // We don't have the line number.
            text: String::new(), // We don't return the text for now.
        }))
    }
}
