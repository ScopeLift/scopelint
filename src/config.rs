use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(version, about, after_help = "Learn more: https://github.com/ScopeLift/scopelint")]
pub struct Opts {
    #[clap(subcommand)]
    pub subcommand: Subcommands,
}

#[derive(Debug, Subcommand)]
pub enum Subcommands {
    #[clap(about = "Checks code to verify all conventions are being followed.")]
    Check,
    #[clap(about = "Formats Solidity and TOML files in the codebase.")]
    Format,
}
