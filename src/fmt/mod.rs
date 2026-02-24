use colored::Colorize;
use std::{error::Error, fs, process};

/// Check formatting without modifying files.
/// # Errors
/// Errors if `forge fmt` fails, or if `taplo` fails to format `foundry.toml`.
fn check_formatting(taplo_opts: taplo::formatter::Options) -> Result<(), Box<dyn Error>> {
    println!("Checking formatting...");

    let forge_status = process::Command::new("forge").args(["fmt", "--check"]).output()?;

    let mut has_changes = false;

    // Print any warnings/errors from `forge fmt --check`.
    if !forge_status.stderr.is_empty() {
        print!("{}", String::from_utf8(forge_status.stderr)?);
    }

    // Print the diff output from forge fmt --check with colors
    if !forge_status.stdout.is_empty() {
        println!("Solidity files that would be reformatted:");
        let forge_output = String::from_utf8(forge_status.stdout)?;

        for line in forge_output.lines() {
            if line.starts_with("Diff in ") {
                println!("{line}");
            } else if line.contains("|-") {
                // Red for removed lines
                let parts: Vec<&str> = line.split("|-").collect();
                if parts.len() == 2 {
                    println!("{}{}{}", parts[0], "|-".red(), parts[1].red());
                } else {
                    println!("{line}");
                }
            } else if line.contains("|+") {
                // Green for added lines
                let parts: Vec<&str> = line.split("|+").collect();
                if parts.len() == 2 {
                    println!("{}{}{}", parts[0], "|+".green(), parts[1].green());
                } else {
                    println!("{line}");
                }
            } else {
                println!("{line}");
            }
        }
        has_changes = true;
    }

    // Check if forge fmt found any issues
    if !forge_status.status.success() {
        has_changes = true;
    }

    // Check foundry.toml formatting
    let config_orig = fs::read_to_string("./foundry.toml")?;
    let config_fmt = taplo::formatter::format(&config_orig, taplo_opts);

    if config_orig != config_fmt {
        println!("foundry.toml would be reformatted:");
        println!("Diff in foundry.toml:");

        // Simple diff output with colors
        let orig_lines: Vec<&str> = config_orig.lines().collect();
        let fmt_lines: Vec<&str> = config_fmt.lines().collect();

        for (i, line) in fmt_lines.iter().enumerate() {
            if i < orig_lines.len() && orig_lines[i] != *line {
                // Red for removed lines
                println!("{}    |{}{}", i + 1, "-".red(), orig_lines[i].red());
                // Green for added lines
                println!("{}    |{}{}", i + 1, "+".green(), line.green());
            } else if i >= orig_lines.len() {
                // Green for new lines
                println!("{}    |{}{}", i + 1, "+".green(), line.green());
            }
        }

        has_changes = true;
    }

    // Exit with error code if any files would be changed
    if has_changes {
        println!("\nRun 'scopelint fmt' to apply these changes.");
        process::exit(1);
    } else {
        println!("All files are properly formatted!");
    }

    Ok(())
}

/// Apply formatting to files.
/// # Errors
/// Errors if `forge fmt` fails, or if `taplo` fails to format `foundry.toml`.
fn apply_formatting(taplo_opts: taplo::formatter::Options) -> Result<(), Box<dyn Error>> {
    let forge_status = process::Command::new("forge").arg("fmt").output()?;

    // Print any warnings/errors from `forge fmt`.
    if !forge_status.stderr.is_empty() {
        print!("{}", String::from_utf8(forge_status.stderr)?);
    }

    // Format `foundry.toml` with taplo.
    let config_orig = fs::read_to_string("./foundry.toml")?;
    let config_fmt = taplo::formatter::format(&config_orig, taplo_opts);
    fs::write("./foundry.toml", config_fmt)?;
    Ok(())
}

/// Format the code.
/// # Errors
/// Errors if `forge fmt` fails, or if `taplo` fails to format `foundry.toml`.
pub fn run(taplo_opts: taplo::formatter::Options, check: bool) -> Result<(), Box<dyn Error>> {
    if check {
        check_formatting(taplo_opts)
    } else {
        apply_formatting(taplo_opts)
    }
}
