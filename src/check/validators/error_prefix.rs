use solang_parser::pt::{ContractPart, ErrorDefinition, SourceUnitPart};

use crate::check::{
    utils::{FileKind, InvalidItem, IsFileKind, ValidatorKind},
    Parsed,
};
#[must_use]
/// Validates that error names are prefixed with `ContractName_`
pub fn validate(parsed: &Parsed) -> Vec<InvalidItem> {
    if !is_matching_file(parsed) {
        return Vec::new();
    }

    let mut invalid_items: Vec<InvalidItem> = Vec::new();

    for element in &parsed.pt.0 {
        if let SourceUnitPart::ContractDefinition(c) = element {
            // Skip contracts without names
            let Some(contract_name) = c.name.as_ref().map(|n| n.name.clone()) else {
                continue;
            };

            for el in &c.parts {
                if let ContractPart::ErrorDefinition(e) = el {
                    if let Some(invalid_item) = validate_name(parsed, e, Some(&contract_name)) {
                        invalid_items.push(invalid_item);
                    }
                }
            }
        }
    }

    invalid_items
}

fn is_matching_file(parsed: &Parsed) -> bool {
    let file = &parsed.file;
    file.is_file_kind(FileKind::Src, &parsed.path_config) ||
        file.is_file_kind(FileKind::Test, &parsed.path_config) ||
        file.is_file_kind(FileKind::Handler, &parsed.path_config)
}

fn validate_name(
    parsed: &Parsed,
    e: &ErrorDefinition,
    contract_name: Option<&str>,
) -> Option<InvalidItem> {
    // Skip errors without names
    let error_info = e.name.as_ref()?;
    let error_name = &error_info.name;
    let error_loc = error_info.loc;

    // If no contract name provided (top-level error), it's valid
    let contract_name = contract_name?;
    let expected_prefix = format!("{contract_name}_");

    if error_name.starts_with(&expected_prefix) {
        None // Valid - error name is prefixed with contract name
    } else {
        Some(InvalidItem::new(
            ValidatorKind::Error,
            parsed,
            error_loc,
            format!("Error '{error_name}' should be prefixed with '{contract_name}_'"),
        ))
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
                // Valid error names (prefixed with contract name)
                error MyContract_ValidError();
                error MyContract_AnotherError(uint256 value);
                
                // Invalid error names (not prefixed with contract name)
                error InvalidError();
                error AnotherInvalidError(uint256 value);
            }
        ";

        let expected_findings =
            ExpectedFindings { src: 2, test: 2, handler: 2, ..ExpectedFindings::default() };
        expected_findings.assert_eq(content, &validate);
    }

    #[test]
    fn test_validate_with_ignore_error_next_line() {
        let content = r"contract MyContract {
    // scopelint: ignore-error-next-line
    error InvalidError();
    
    // This one should still be flagged
    error AnotherInvalidError(uint256 value);
}";

        // Only one error should be found (the one without ignore directive)
        let expected_findings =
            ExpectedFindings { src: 1, test: 1, handler: 1, ..ExpectedFindings::default() };
        expected_findings.assert_eq(content, &validate);
    }

    #[test]
    fn test_validate_with_ignore_error_start_end() {
        let content = r"contract MyContract {
    // scopelint: ignore-error-start
    error InvalidError();
    error AnotherInvalidError(uint256 value);
    // scopelint: ignore-error-end
    
    // This one should still be flagged
    error YetAnotherInvalidError(uint256 value);
}";

        // Only one error should be found (outside the ignore region)
        let expected_findings =
            ExpectedFindings { src: 1, test: 1, handler: 1, ..ExpectedFindings::default() };
        expected_findings.assert_eq(content, &validate);
    }

    #[test]
    fn test_validate_with_ignore_error_file() {
        let content = r"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.17;
// scopelint: ignore-error-file

contract MyContract {
    error InvalidError();
    error AnotherInvalidError(uint256 value);
    error YetAnotherInvalidError(uint256 value);
}";

        // All errors should be ignored for the entire file
        let expected_findings = ExpectedFindings::new(0);
        expected_findings.assert_eq(content, &validate);
    }
}
