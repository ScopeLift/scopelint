use crate::check::{
    report, utils,
    utils::{Name, Validate},
};
use once_cell::sync::Lazy;
use regex::Regex;
use solang_parser::pt::{ContractPart, FunctionDefinition, SourceUnitPart};
use std::{error::Error, fs, path::Path};

// A regex matching valid test names, see the `validate_test_names_regex` test for examples.
static RE_VALID_TEST_NAME: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^test(Fork)?(Fuzz)?(_Revert(If|When|On))?_(\w+)*$").unwrap());

impl utils::Validate for FunctionDefinition {
    fn validate(&self, content: &str, file: &Path) -> Option<report::InvalidItem> {
        let name = &self.name();

        if file.starts_with("./test") && !is_valid_test_name(name) {
            return Some(report::InvalidItem::new(
                report::Validator::Test,
                file.display().to_string(),
                name.to_string(),
                utils::offset_to_line(content, self.loc.start()),
            ))
        }
        None
    }
}

pub fn run() -> Result<Vec<report::InvalidItem>, Box<dyn Error>> {
    let mut invalid_items = Vec::new();

    let files = utils::get_files(&utils::FileKind::TestContracts)?;
    for file in files {
        let content = fs::read_to_string(&file)?;
        let (pt, _comments) = solang_parser::parse(&content, 0).expect("Parsing failed");
        // Run checks.
        for element in pt.0 {
            match element {
                SourceUnitPart::FunctionDefinition(f) => {
                    invalid_items.extend(f.validate(&content, &file));
                }
                SourceUnitPart::ContractDefinition(c) => {
                    for el in c.parts {
                        if let ContractPart::FunctionDefinition(f) = el {
                            invalid_items.extend(f.validate(&content, &file));
                        }
                    }
                }
                _ => (),
            }
        }
    }

    Ok(invalid_items)
}

fn is_valid_test_name(name: &str) -> bool {
    if !name.starts_with("test") {
        return true // Not a test function, so return.
    }
    RE_VALID_TEST_NAME.is_match(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_test_names_regex() {
        let allowed_names = vec![
            "test_Description",
            "test_Increment",
            "testFuzz_Description",
            "testFork_Description",
            "testForkFuzz_Description",
            "testForkFuzz_Description_MoreInfo",
            "test_RevertIf_Condition",
            "test_RevertWhen_Condition",
            "test_RevertOn_Condition",
            "test_RevertOn_Condition_MoreInfo",
            "testFuzz_RevertIf_Condition",
            "testFuzz_RevertWhen_Condition",
            "testFuzz_RevertOn_Condition",
            "testFuzz_RevertOn_Condition_MoreInfo",
            "testForkFuzz_RevertIf_Condition",
            "testForkFuzz_RevertWhen_Condition",
            "testForkFuzz_RevertOn_Condition",
            "testForkFuzz_RevertOn_Condition_MoreInfo",
            "testForkFuzz_RevertOn_Condition_MoreInfo_Wow",
            "testForkFuzz_RevertOn_Condition_MoreInfo_Wow_As_Many_Underscores_As_You_Want",
        ];

        let disallowed_names = [
            "test",
            "testDescription",
            "testDescriptionMoreInfo",
            // TODO The below are tough to prevent without regex look-ahead support.
            // "test_RevertIfCondition",
            // "test_RevertWhenCondition",
            // "test_RevertOnCondition",
            // "testFuzz_RevertIfDescription",
            // "testFuzz_RevertWhenDescription",
            // "testFuzz_RevertOnDescription",
            // "testForkFuzz_RevertIfCondition",
            // "testForkFuzz_RevertWhenCondition",
            // "testForkFuzz_RevertOnCondition",
        ];

        for name in allowed_names {
            assert_eq!(is_valid_test_name(name), true, "{name}");
        }

        for name in disallowed_names {
            assert_eq!(is_valid_test_name(name), false, "{name}");
        }
    }
}
