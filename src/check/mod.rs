use crate::check::{comments::Comments, inline_config::InlineConfig};
use colored::Colorize;
use itertools::Itertools;
use solang_parser::helpers::OptionalCodeLocation;
use std::{error::Error, ffi::OsStr, fs};
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
        return Err("Invalid names found".into())
    }
    Ok(())
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
                    continue
                }
            };

            if !dent.file_type().is_file() || dent.path().extension() != Some(OsStr::new("sol")) {
                continue
            }

            // Get the parse tree (pt) of the file and extract inline configs.
            let file = dent.path();
            let src = &fs::read_to_string(file)?;
            let (pt, comments) = solang_parser::parse(src, 0).expect("Parsing failed");
            let comments = Comments::new(comments, src);
            let (inline_config_items, invalid_inline_config_items): (Vec<_>, Vec<_>) =
                comments.parse_inline_config_items().partition_result();
            let inline_config = InlineConfig::new(inline_config_items, src);

            // If there are any invalid inline config items, add them to the results.
            for invalid_item in invalid_inline_config_items {
                results.add_item(utils::InvalidItem::new(
                    utils::ValidatorKind::Directive,
                    file.display().to_string(),
                    invalid_item.1.to_string(),
                    utils::offset_to_line(src, invalid_item.0.start()),
                ));
            }

            // Skip if we're in a disabled region.
            let source_units = &pt.0;
            let is_in_disabled_region = source_units.iter().any(|source_unit| {
                source_unit.loc_opt().map_or(false, |loc| inline_config.is_disabled(loc))
            });
            if is_in_disabled_region {
                continue
            }

            // Run all checks.
            results.add_items(validators::test_names::validate(file, src, &pt));
            results.add_items(validators::src_names_internal::validate(file, src, &pt));
            results.add_items(validators::script_one_pubic_run_method::validate(file, src, &pt));
            results.add_items(validators::constant_names::validate(file, src, &pt));
        }
    }
    Ok(results)
}
