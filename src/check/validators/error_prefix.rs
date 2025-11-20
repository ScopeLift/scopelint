use solang_parser::pt::{ContractPart, ErrorDefinition, SourceUnitPart};

use crate::check::{
    utils::{FileKind, InvalidItem, IsFileKind, ValidatorKind},
    Parsed,
};
use std::path::Path;

#[must_use]
/// Validates that error names are prefixed with `ContractName_`
pub fn validate(parsed: &Parsed) -> Vec<InvalidItem> {
    if !is_matching_file(&parsed.file) {
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

fn is_matching_file(file: &Path) -> bool {
    file.is_file_kind(FileKind::Src) ||
        file.is_file_kind(FileKind::Test) ||
        file.is_file_kind(FileKind::Handler)
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
}
