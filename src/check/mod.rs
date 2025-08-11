use crate::check::{
    comments::Comments,
    inline_config::{InlineConfig, InvalidInlineConfigItem},
};
use colored::Colorize;
use itertools::Itertools;
use solang_parser::pt::{Loc, SourceUnit};
use std::{
    error::Error,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

/// Contains all the types and methods to parse comments.
pub mod comments;

/// Contains all the types and methods to define and parse inline config items.
pub mod inline_config;

/// Contains all the types and methods to generate a report of all the invalid items found.
pub mod report;

/// Contains helper methods, traits, etc. used by the validators and report generation.
pub mod utils;

/// Contains all the validators to ensure Solidity files follow conventions and best practices.
pub mod validators;

/// Validates the code formatting, and print details on any conventions that are not being followed.
/// # Errors
/// Returns an error if the formatting or convention validations fail.
pub fn run(taplo_opts: taplo::formatter::Options) -> Result<(), Box<dyn Error>> {
    // We run the formatting check separate to just indicate whether or not the user needs to format
    // the codebase, whereas the other validators return granular information about what to fix
    // since they currently can't be fixed automatically.
    let valid_names = validate_conventions();
    let valid_fmt = validators::formatting::validate(taplo_opts);

    if valid_names.is_ok() && valid_fmt.is_ok() {
        Ok(())
    } else {
        Err("One or more checks failed, review above output".into())
    }
}

// =============================
// ======== Validations ========
// =============================

fn validate_conventions() -> Result<(), Box<dyn Error>> {
    let paths = ["./src", "./script", "./test"];
    let results = validate(paths)?;

    if !results.is_valid() {
        eprint!("{results}");
        eprintln!("{}: Convention checks failed, see details above", "error".bold().red());
        return Err("Invalid names found".into());
    }
    Ok(())
}

/// Result of parsing the source code. This is the same struct used in forge's fmt module.
#[derive(Debug)]
pub struct Parsed {
    /// Path to the file.
    pub file: PathBuf,
    /// The original source code.
    pub src: String,
    /// The Parse Tree via [`solang_parser`].
    pub pt: SourceUnit,
    /// Parsed comments.
    pub comments: Comments,
    /// Parsed inline config.
    pub inline_config: InlineConfig,
    /// Invalid inline config items parsed.
    pub invalid_inline_config_items: Vec<(Loc, InvalidInlineConfigItem)>,
}

/// Parses the source code and returns a [`Parsed`] struct.
///
/// # Errors
///
/// Returns an error if the file cannot be read or its source code cannot be parsed.
pub fn parse(file: &Path) -> Result<Parsed, Box<dyn Error>> {
    let src = &fs::read_to_string(file)?;

    let (pt, comments) = solang_parser::parse(src, 0).map_err(|d| {
        eprintln!("{d:?}");
        "Failed to parse file".to_string()
    })?;

    let comments = Comments::new(comments, src);
    let (inline_config_items, invalid_inline_config_items): (Vec<_>, Vec<_>) =
        comments.parse_inline_config_items().partition_result();
    let inline_config = InlineConfig::new(inline_config_items, src);

    Ok(Parsed {
        file: file.to_owned(),
        src: src.clone(),
        pt,
        comments,
        inline_config,
        invalid_inline_config_items,
    })
}

// Core validation method that walks the directory and validates all Solidity files.
fn validate(paths: [&str; 3]) -> Result<report::Report, Box<dyn Error>> {
    let mut results = report::Report::default();

    for path in paths {
        for result in WalkDir::new(path) {
            let dent = match result {
                Ok(dent) => dent,
                Err(err) => {
                    eprintln!("{err}");
                    continue;
                }
            };

            if !dent.file_type().is_file() || dent.path().extension() != Some(OsStr::new("sol")) {
                continue;
            }

            // Get the parse tree (pt) of the file and extract inline configs.
            let parsed = parse(dent.path())?;

            // If there are any invalid inline config items, add them to the results.
            for invalid_item in &parsed.invalid_inline_config_items {
                results.add_item(utils::InvalidItem::new(
                    utils::ValidatorKind::Directive,
                    &parsed,
                    invalid_item.0,
                    invalid_item.1.to_string(),
                ));
            }

            // Run all checks.
            results.add_items(validators::test_names::validate(&parsed));
            results.add_items(validators::src_names_internal::validate(&parsed));
            results.add_items(validators::script_has_public_run_method::validate(&parsed));
            results.add_items(validators::constant_names::validate(&parsed));
            results.add_items(validators::src_spdx_header::validate(&parsed));
            results.add_items(validators::variable_names::validate(&parsed));
            results.add_items(validators::event_prefix::validate(&parsed));
        }
    }
    Ok(results)
}
