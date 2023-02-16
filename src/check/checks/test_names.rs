use crate::check::{
    report::{InvalidItem, Validator},
    utils::{self, FileKind, IsFileKind, Name},
};
use once_cell::sync::Lazy;
use regex::Regex;
use solang_parser::pt::{ContractPart, SourceUnit, SourceUnitPart};
use std::{error::Error, path::Path};

// A regex matching valid test names, see the `validate_test_names_regex` test for examples.
static RE_VALID_TEST_NAME: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^test(Fork)?(Fuzz)?(_Revert(If|When|On))?_(\w+)*$").unwrap());

pub fn validate(
    file: &Path,
    content: &str,
    pt: &SourceUnit,
) -> Result<Vec<InvalidItem>, Box<dyn Error>> {
    if !file.is_file_kind(FileKind::TestContracts) {
        return Ok(Vec::new())
    }

    let mut invalid_items: Vec<InvalidItem> = Vec::new();
    for element in &pt.0 {
        match element {
            SourceUnitPart::FunctionDefinition(f) => {
                let name = f.name();
                if !is_valid_test_name(&name) {
                    invalid_items.push(InvalidItem::new(
                        Validator::Test,
                        file.display().to_string(),
                        name.to_string(),
                        utils::offset_to_line(content, f.loc.start()),
                    ));
                }
            }
            SourceUnitPart::ContractDefinition(c) => {
                for el in &c.parts {
                    if let ContractPart::FunctionDefinition(f) = el {
                        let name = f.name();
                        if !is_valid_test_name(&name) {
                            invalid_items.push(InvalidItem::new(
                                Validator::Test,
                                file.display().to_string(),
                                name.to_string(),
                                utils::offset_to_line(content, f.loc.start()),
                            ));
                        }
                    }
                }
            }
            _ => (),
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
