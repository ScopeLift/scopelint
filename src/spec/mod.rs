use std::error::Error;

/// Generates a specification for the current project from test names.
/// # Errors
/// Returns an error if the specification could not be generated from the Solidity code.
pub fn run() -> Result<(), Box<dyn Error>> {
    println!("Generating specification...");
    Ok(())
}
