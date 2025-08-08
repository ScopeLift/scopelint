use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(version, about, after_help = "Learn more: https://github.com/ScopeLift/scopelint")]
/// Options for running scopelint.
pub struct Opts {
    #[clap(subcommand)]
    /// The mode to run scopelint in.
    pub subcommand: Subcommands,
}

#[derive(Debug, Subcommand)]
/// The mode to run scopelint in.
pub enum Subcommands {
    #[clap(about = "Checks code to verify all conventions are being followed.")]
    /// Checks code to verify all conventions are being followed.
    Check,
    #[clap(about = "Formats Solidity and TOML files in the codebase.")]
    /// Formats Solidity and TOML files in the codebase.
    Fmt {
        #[clap(long, help = "Show changes without modifying files")]
        /// Show changes without modifying files.
        check: bool,
    },
    #[clap(about = "Generates a specification for the current project from test names.")]
    /// Generates a specification for the current project from test names.
    Spec,
}
