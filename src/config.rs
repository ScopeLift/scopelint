use clap::{Parser, Subcommand};

/// Returns version information with appropriate suffix
fn version_info() -> &'static str {
    // For local development builds, add -dev suffix
    if cfg!(debug_assertions) {
        return concat!(env!("CARGO_PKG_VERSION"), "-dev");
    }

    // For release builds, check if this is a beta release
    // We'll use a build-time environment variable to set this
    if option_env!("GIT_TAG") == Some("beta") {
        return concat!(env!("CARGO_PKG_VERSION"), "-beta");
    }

    // For release builds, use the version as-is
    env!("CARGO_PKG_VERSION")
}

#[derive(Debug, Parser)]
#[clap(version = version_info(), about, after_help = "Learn more: https://github.com/ScopeLift/scopelint")]
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
    Spec {
        #[clap(long, help = "Show internal functions in the specification.")]
        /// Show internal functions in the specification.
        show_internal: bool,
    },
}
