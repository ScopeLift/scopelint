use solang_parser::pt::{ContractPart, EventDefinition, SourceUnitPart};

use crate::check::{
    utils::{FileKind, InvalidItem, IsFileKind, ValidatorKind},
    Parsed,
};
use std::path::Path;

#[must_use]
/// Validates that event names are prefixed with `ContractName_`
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
                if let ContractPart::EventDefinition(e) = el {
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
    file.is_file_kind(FileKind::Src) || file.is_file_kind(FileKind::Test)
}

fn validate_name(
    parsed: &Parsed,
    e: &EventDefinition,
    contract_name: Option<&str>,
) -> Option<InvalidItem> {
    // Skip events without names
    let event_info = e.name.as_ref()?;
    let event_name = &event_info.name;
    let event_loc = event_info.loc;

    // If no contract name provided (top-level event), it's valid
    let contract_name = contract_name?;
    let expected_prefix = format!("{contract_name}_");

    if event_name.starts_with(&expected_prefix) {
        None // Valid - event name is prefixed with contract name
    } else {
        Some(InvalidItem::new(
            ValidatorKind::Event,
            parsed,
            event_loc,
            format!("Event '{event_name}' should be prefixed with '{contract_name}_'"),
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
                // Valid event names (prefixed with contract name)
                event MyContract_ValidEvent();
                event MyContract_AnotherEvent(uint256 value);
                
                // Invalid event names (not prefixed with contract name)
                event InvalidEvent();
                event AnotherInvalidEvent(uint256 value);
            }
        ";

        let expected_findings = ExpectedFindings { src: 2, test: 2, ..ExpectedFindings::default() };
        expected_findings.assert_eq(content, &validate);
    }
}
