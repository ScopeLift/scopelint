use std::{error::Error, fs, process};

/// Format the code.
/// # Errors
/// Errors if `forge fmt` fails, or if `taplo` fails to format `foundry.toml`.
pub fn run(taplo_opts: taplo::formatter::Options) -> Result<(), Box<dyn Error>> {
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
    Ok(())
}
