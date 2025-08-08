use colored::Colorize;
use std::{error::Error, fs, process};

/// Validates that Solidity and TOML files are formatted correctly.
/// # Errors
/// Returns an error if formatting is invalid or parsing fails.
pub fn validate(taplo_opts: taplo::formatter::Options) -> Result<(), Box<dyn Error>> {
    // Check Solidity with `forge fmt`.
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
        return Err("Invalid fmt found".into());
    }
    Ok(())
}
