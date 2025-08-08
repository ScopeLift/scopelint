use crate::check::{
    utils::{FileKind, InvalidItem, IsFileKind, Name, ValidatorKind, VisibilitySummary},
    Parsed,
};
use regex::Regex;
use solang_parser::pt::{ContractPart, FunctionDefinition, SourceUnitPart};
use std::{path::Path, sync::LazyLock};

// A regex matching valid test names, see the `validate_test_names_regex` test for examples.
static RE_VALID_TEST_NAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^test(Fork)?(Fuzz)?(_Revert(If|When|On|Given))?_(\w+)*$").unwrap()
});

fn is_matching_file(file: &Path) -> bool {
    file.is_file_kind(FileKind::Test)
}

#[must_use]
/// Validates that test names are in the correct format.
pub fn validate(parsed: &Parsed) -> Vec<InvalidItem> {
    if !is_matching_file(&parsed.file) {
        return Vec::new();
    }

    let mut invalid_items: Vec<InvalidItem> = Vec::new();
    for element in &parsed.pt.0 {
        match element {
            SourceUnitPart::FunctionDefinition(f) => {
                if let Some(invalid_item) = validate_name(parsed, f) {
                    invalid_items.push(invalid_item);
                }
            }
            SourceUnitPart::ContractDefinition(c) => {
                for el in &c.parts {
                    if let ContractPart::FunctionDefinition(f) = el {
                        if let Some(invalid_item) = validate_name(parsed, f) {
                            invalid_items.push(invalid_item);
                        }
                    }
                }
            }
            _ => (),
        }
    }
    invalid_items
}

fn is_valid_test_name(name: &str) -> bool {
    // Check that name matches the allowed pattern.
    if !name.starts_with("test") || !RE_VALID_TEST_NAME.is_match(name) {
        return false;
    }

    // Verify the revert naming convention. This is a workaround for the regex create not supporting
    // look-ahead/behind. We could use the `fancy_regex` crate, but the regex does get complicated
    // and may be hard to understand and maintain, which is why we use this simpler regex + segment
    // parsing approach.
    let segments: Vec<&str> = name.split('_').collect();
    for segment in segments {
        // If the segment contains `Revert` but does not start with `Revert` it is invalid.
        if segment.contains("Revert") && !segment.starts_with("Revert") {
            return false;
        }

        // If the segment starts with `Revert` it is valid if the rest of the segment is exactly
        // `If`, `When`, `On`, or `Given`.
        if segment.starts_with("Revert") {
            match segment.strip_prefix("Revert") {
                Some("If" | "When" | "On" | "Given") => {}
                _ => return false,
            }
        }
    }

    true
}

fn is_test_function(f: &FunctionDefinition) -> bool {
    f.is_public_or_external() && f.name().starts_with("test")
}

fn validate_name(parsed: &Parsed, f: &FunctionDefinition) -> Option<InvalidItem> {
    let name = f.name();
    if is_test_function(f) && !is_valid_test_name(&name) {
        Some(InvalidItem::new(ValidatorKind::Test, parsed, f.name_loc, name))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::utils::ExpectedFindings;

    #[test]
    fn test_validate() {
        let content = r"
            contract MyContract {
                // Good test names.
                function test_Description() public {}
                function test_Increment() public {}
                function testFuzz_Description() external {}
                function testFork_Description() external {}

                // Bad test names.
                function test() public {}
                function testDescription() public {}
                function testDescriptionMoreInfo() external {}

                // Things that are not tests and should be ignored.
                function test() internal {}
                function testDescription() internal {}
                function testDescriptionMoreInfo() private {}

                function _test() public {}
                function _testDescription() public {}
                function _testDescriptionMoreInfo() public {}
            }
        ";

        let expected_findings = ExpectedFindings { test: 3, ..ExpectedFindings::default() };
        expected_findings.assert_eq(content, &validate);
    }

    #[test]
    fn test_is_valid_test_name() {
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
            "testFuzz_RevertGiven_Condition",
            "testForkFuzz_RevertIf_Condition",
            "testForkFuzz_RevertWhen_Condition",
            "testForkFuzz_RevertOn_Condition",
            "testForkFuzz_RevertGiven_Condition",
            "testForkFuzz_RevertOn_Condition_MoreInfo",
            "testForkFuzz_RevertOn_Condition_MoreInfo_Wow",
            "testForkFuzz_RevertOn_Condition_MoreInfo_Wow_As_Many_Underscores_As_You_Want",
        ];

        let disallowed_names = [
            "test",
            "test123_Description",
            "testDescription",
            "testDescriptionMoreInfo",
            "testRevertIfCondition",
            "testRevertIf_Condition",
            "test_RevertIfCondition",
            "test_RevertWhenCondition",
            "test_RevertOnCondition",
            "test_RevertGivenCondition",
            "testFuzz_RevertIfDescription",
            "testFuzz_RevertWhenDescription",
            "testFuzz_RevertGivenDescription",
            "testFuzz_RevertOnDescription",
            "testForkFuzz_RevertIfCondition",
            "testForkFuzz_RevertWhenCondition",
            "testForkFuzz_RevertOnCondition",
            "testForkFuzz_RevertGivenCondition",
        ];

        for name in allowed_names {
            assert!(is_valid_test_name(name), "{name}");
        }

        for name in disallowed_names {
            assert!(!is_valid_test_name(name), "{name}");
        }
    }
}
