#![doc = include_str!("../README.md")]
#![warn(unreachable_pub, unused, rust_2021_compatibility)]
#![warn(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]
#![allow(clippy::multiple_crate_versions)]
use std::error::Error;

/// Runs validators on Solidity files.
pub mod check;

/// Parses library configuration.
pub mod config;

/// Parses Solidity source files.
pub mod parser;

/// Formats Solidity and TOML files.
pub mod fmt;

/// Generates a specification for the current project from test names.
pub mod spec;

// ===========================
// ======== Execution ========
// ===========================

/// Takes the provided `opts` and runs the program.
/// # Errors
/// Errors if the provided mode fails to run.
pub fn run(opts: &config::Opts) -> Result<(), Box<dyn Error>> {
    // Configure formatting options, https://taplo.tamasfe.dev/.
    let taplo_opts = taplo::formatter::Options {
        allowed_blank_lines: 1,
        indent_entries: true,
        reorder_keys: true,
        ..Default::default()
    };

    // Execute commands.
    match &opts.subcommand {
        config::Subcommands::Check => check::run(taplo_opts),
        config::Subcommands::Fmt { check } => fmt::run(taplo_opts, *check),
        config::Subcommands::Spec { show_internal } => spec::run(*show_internal),
    }
}
