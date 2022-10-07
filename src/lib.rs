use grep::{
    matcher::Matcher,
    regex::RegexMatcher,
    searcher::{sinks::UTF8, BinaryDetection, SearcherBuilder},
};
use regex::Regex;
use std::{error::Error, fs, process};
use walkdir::WalkDir;

// Using this enum and struct to simplify future changes if we allow more
// granularity, though this is probably overkill, especially since we'd likely
// use clap if input arguments get more complex.
enum Mode {
    Format,
    Check,
}

/// Program configuration. Valid modes are `fmt` and `check`.
pub struct Config {
    mode: Mode,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() > 2 {
            return Err("Too many arguments")
        }

        let mode = match args.len() {
            1 => Mode::Format, // Default to format if no args provided.
            2 => match args[1].as_str() {
                "fmt" => Mode::Format,
                "check" => Mode::Check,
                _ => panic!("Invalid argument {}", &args[1]),
            },
            _ => panic!("Too many arguments"),
        };
        Ok(Config { mode })
    }
}

/// Takes the provided `config` and runs the program.
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
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
    }
}

fn fmt(taplo_opts: taplo::formatter::Options) -> Result<(), Box<dyn Error>> {
    // Format Solidity with forge
    process::Command::new("forge")
        .arg("fmt")
        .output()
        .expect("forge fmt failed");

    // Format `foundry.toml` with taplo.
    let config_orig = fs::read_to_string("./foundry.toml")?;
    let config_fmt = taplo::formatter::format(&config_orig, taplo_opts);
    fs::write("./foundry.toml", config_fmt)?;

    // Check test names.
    validate_names()
}

fn check(taplo_opts: taplo::formatter::Options) -> Result<(), Box<dyn Error>> {
    // Check Solidity with `forge fmt`
    let forge_status =
        process::Command::new("forge").arg("fmt").arg("--check").output()?;
    let forge_ok = forge_status.status.success();

    // Check TOML with `taplo fmt`
    let config_orig = fs::read_to_string("./foundry.toml")?;
    let config_fmt = taplo::formatter::format(&config_orig, taplo_opts);
    let taplo_ok = config_orig == config_fmt;

    // Check naming conventions.
    let valid_names = validate_names();

    // Log results and exit.
    if !forge_ok || !taplo_ok {
        eprintln!("Error: Formatting failed, run `scopelint fmt` to fix");
    }

    if forge_ok && taplo_ok && valid_names.is_ok() {
        Ok(())
    } else {
        Err("One or more checks failed, review above output".into())
    }
}

fn validate_names() -> Result<(), Box<dyn Error>> {
    let paths = ["./src", "./script", "./test"];

    let pattern = r"\sfunction\stest\w{1,}\(";
    let test_names_ok = search_test_names(pattern, paths)?;

    let pattern = r"\sconstant\s";
    let constant_names_ok = search_constant_names(pattern, paths)?;

    if test_names_ok && constant_names_ok {
        Ok(())
    } else {
        eprintln!("Error: Invalid names found, see details above");
        Err("Invalid names found".into())
    }
}

struct Match {
    file: String, // File name.
    line: u64,    // Line number.
    text: String, // Incorrectly named item.
}

// Reference: https://github.com/BurntSushi/ripgrep/blob/master/crates/grep/examples/simplegrep.rs
fn search_test_names(
    pattern: &str,
    paths: [&str; 3],
) -> Result<bool, Box<dyn Error>> {
    let mut success = true; // Default to true.
    let test_matcher = RegexMatcher::new_line_matcher(pattern)?;

    let mut searcher = SearcherBuilder::new()
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .line_number(true)
        .build();

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

            let mut misnamed_tests: Vec<Match> = vec![];
            searcher.search_path(
                &test_matcher,
                dent.path(),
                UTF8(|lnum, line| {
                    // We are guaranteed to find a match, so the unwrap is ok.
                    let test_match =
                        test_matcher.find(line.as_bytes())?.unwrap();
                    let test_name = line[test_match].to_string();

                    // Found a test, check if it matches our pattern.
                    let test_validator = RegexMatcher::new_line_matcher(
                        r"test(Fork)?(Fuzz)?_(Revert(If_|When_){1})?\w{1,}\(",
                    )
                    .expect("Could not create regex matcher");

                    // If match is found, test name is good, otherwise it's bad.
                    let test_result = test_validator.find(test_name.as_bytes());
                    if test_result?.is_none() {
                        misnamed_tests.push(Match {
                            file: dent.path().to_str().unwrap().to_string(),
                            line: lnum,
                            text: test_name,
                        });
                    }
                    Ok(true)
                }),
            )?;

            success = misnamed_tests.is_empty();
            for test in misnamed_tests {
                let trimmed_test = test.text.trim();
                eprintln!(
                    "Misnamed test found in {file} on line {line}: {text}",
                    file = test.file,
                    line = test.line,
                    // Start at index 9 to remove "function ", and end at -1 to
                    // remove the closing parenthesis.
                    text = &trimmed_test[9..trimmed_test.len() - 1]
                );
            }
        }
    }

    Ok(success)
}

fn search_constant_names(
    pattern: &str,
    paths: [&str; 3],
) -> Result<bool, Box<dyn Error>> {
    let mut success = true; // Default to true.
    let test_matcher = RegexMatcher::new_line_matcher(pattern)?;

    let mut searcher = SearcherBuilder::new()
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .line_number(true)
        .build();

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

            let mut misnamed_vars: Vec<Match> = vec![];
            searcher.search_path(
                &test_matcher,
                dent.path(),
                UTF8(|lnum, line| {
                    // Found a constant/immutable, get the var name.
                    let r = Regex::new(r"(;|=)").unwrap();
                    let mut split_str = r.split(line);
                    let var = split_str
                        .next()
                        .expect("no match")
                        .split_whitespace()
                        .last()
                        .expect("no match");

                    // Make sure it's ALL_CAPS: https://regex101.com/r/Pv9mD8/1
                    let name_validator = RegexMatcher::new_line_matcher(
                        r"^[A-Z]+(?:_{0,1}[A-Z]+)*$",
                    )
                    .expect("Could not create regex matcher");

                    // If match is found, test name is good, otherwise it's bad.
                    let test_result = name_validator.find(var.as_bytes());
                    if test_result?.is_none() {
                        misnamed_vars.push(Match {
                            file: dent.path().to_str().unwrap().to_string(),
                            line: lnum,
                            text: var.to_string(),
                        });
                    }
                    Ok(true)
                }),
            )?;

            success = misnamed_vars.is_empty();
            for var in misnamed_vars {
                eprintln!(
                    "Misnamed constant or immutable found in {file} on line {line}: {text}",
                    file = var.file,
                    line = var.line,
                    text = var.text
                );
            }
        }
    }

    Ok(success)
}
