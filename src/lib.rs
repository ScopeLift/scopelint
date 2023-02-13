#![doc = include_str!("../README.md")]
#![warn(unreachable_pub, unused, rust_2021_compatibility)]
#![warn(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]

use std::error::Error;

pub mod check;
pub mod config;
pub mod fmt;
pub mod version;

// ===========================
// ======== Execution ========
// ===========================

/// Takes the provided `config` and runs the program.
/// # Errors
/// Errors if the provided mode fails to run.
pub fn run(config: &config::Config) -> Result<(), Box<dyn Error>> {
    // Configure formatting options, https://taplo.tamasfe.dev/.
    let taplo_opts = taplo::formatter::Options {
        allowed_blank_lines: 1,
        indent_entries: true,
        reorder_keys: true,
        ..Default::default()
    };

    // Execute commands.
    match config.mode {
        config::Mode::Format => fmt::run(taplo_opts),
        config::Mode::Check => check::run(taplo_opts),
        config::Mode::Version => {
            version::run();
            Ok(())
        }
    }
}
