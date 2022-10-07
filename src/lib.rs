use grep::matcher::Matcher;
use grep::regex::RegexMatcher;
use grep::searcher::sinks::UTF8;
use grep::searcher::{BinaryDetection, SearcherBuilder};
use std::error::Error;
use std::{fs, process};
use walkdir::WalkDir;

// For now our config is very simple and this enum/struct is arguably overkill,
// but doing this to simplify future changes if we allow more granularity.
enum Mode {
    Format,
    Check,
}

pub struct Config {
    mode: Mode,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() > 2 {
            return Err("Too many arguments");
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

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    // Configure formatting options, overwriting only a few of the defaults.
    // https://taplo.tamasfe.dev/.
    let mut taplo_opts = taplo::formatter::Options::default();
    taplo_opts.allowed_blank_lines = 1;
    taplo_opts.indent_entries = true;
    taplo_opts.reorder_keys = true;

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
    validate_test_names()
}

fn check(taplo_opts: taplo::formatter::Options) -> Result<(), Box<dyn Error>> {
    // Check Solidity with `forge fmt`
    let forge_status = process::Command::new("forge")
        .arg("fmt")
        .arg("--check")
        .output()?;
    let forge_ok = forge_status.status.success();

    // Check TOML with `taplo fmt`
    let config_orig = fs::read_to_string("./foundry.toml")?;
    let config_fmt = taplo::formatter::format(&config_orig, taplo_opts);
    let taplo_ok = config_orig == config_fmt;

    // Check test names.
    let valid_test_names = validate_test_names();

    // Log results and exit.
    if !forge_ok || !taplo_ok {
        eprintln!("Error: Formatting failed, run `scopelint fmt` to fix");
    }

    if valid_test_names.is_err() {
        eprintln!("Error: Invalid test names, see details above");
    }

    if forge_ok && taplo_ok && valid_test_names.is_ok() {
        Ok(())
    } else {
        Err("One or more checks failed, review above output".into())
    }
}

fn validate_test_names() -> Result<(), Box<dyn Error>> {
    let pattern = r"\sfunction\stest\w{1,}\(";
    let ok_src = search(pattern, &"./src").expect("src search failed");
    let ok_script = search(pattern, &"./script").expect("script search failed");
    let ok_test = search(pattern, &"./test").expect("test search failed");

    if ok_src && ok_script && ok_test {
        Ok(())
    } else {
        Err("Invalid test names".into())
    }
}

// Reference: https://github.com/BurntSushi/ripgrep/blob/master/crates/grep/examples/simplegrep.rs
fn search(pattern: &str, path: &str) -> Result<bool, Box<dyn Error>> {
    struct Match {
        file: String, // File name.
        line: u64,    // Line number.
        text: String, // Test name.
    }

    let mut success = true; // Default to true.
    let test_matcher = RegexMatcher::new_line_matcher(&pattern)?;

    let mut searcher = SearcherBuilder::new()
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .line_number(true)
        .build();

    for result in WalkDir::new(path) {
        let dent = match result {
            Ok(dent) => dent,
            Err(err) => {
                eprintln!("{err}");
                continue;
            }
        };

        if !dent.file_type().is_file() {
            continue;
        }

        let mut misnamed_tests: Vec<Match> = vec![];
        searcher.search_path(
            &test_matcher,
            dent.path(),
            UTF8(|lnum, line| {
                // We are guaranteed to find a match, so the unwrap is ok.
                let test_match = test_matcher.find(line.as_bytes())?.unwrap();
                let test_name = line[test_match].to_string();

                // Now that we found a test, we check if it matches our pattern.
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
                // Start at index 9 to remove "function ", and end at -1 to remove the closing parenthesis.
                text = trimmed_test[9..trimmed_test.len() - 1].to_string()
            );
        }
    }

    Ok(success)
}
