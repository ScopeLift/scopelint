/// Available modes to run the program in.
pub enum Mode {
    /// Formats the code.
    Format,
    /// Checks for conventions not being follows.
    Check,
    /// Prints the version.
    Version,
}

/// Program configuration. Valid modes are `fmt`, `check`, and `--version`.
pub struct Config {
    /// The mode to run the program in.
    pub mode: Mode,
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
