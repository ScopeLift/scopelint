use crate::check::{
    utils::{InvalidItem, ValidatorKind},
    Parsed,
};
use regex::Regex;
use solang_parser::pt::{ContractPart, SourceUnitPart, VariableAttribute, VariableDefinition};
use std::{path::Path, sync::LazyLock};

// A regex matching valid constant names, see the `validate_constant_names_regex` test for examples.
static RE_VALID_CONSTANT_NAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?:[$_]*[A-Z0-9][$_]*){1,}$").unwrap());

const fn is_matching_file(_file: &Path) -> bool {
    true
}

#[must_use]
/// Validates that constant and immutable variable names are in `ALL_CAPS`.
pub fn validate(parsed: &Parsed) -> Vec<InvalidItem> {
    if !is_matching_file(&parsed.file) {
        return Vec::new();
    }

    let mut invalid_items: Vec<InvalidItem> = Vec::new();
    for element in &parsed.pt.0 {
        match element {
            SourceUnitPart::VariableDefinition(v) => {
                if let Some(invalid_item) = validate_name(parsed, v) {
                    invalid_items.push(invalid_item);
                }
            }
            SourceUnitPart::ContractDefinition(c) => {
                for el in &c.parts {
                    if let ContractPart::VariableDefinition(v) = el {
                        if let Some(invalid_item) = validate_name(parsed, v) {
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

fn is_valid_constant_name(name: &str) -> bool {
    RE_VALID_CONSTANT_NAME.is_match(name)
}

fn validate_name(parsed: &Parsed, v: &VariableDefinition) -> Option<InvalidItem> {
    let is_constant = v
        .attrs
        .iter()
        .any(|a| matches!(a, VariableAttribute::Constant(_) | VariableAttribute::Immutable(_)));

    if !is_constant {
        return None;
    }

    v.name.as_ref().and_then(|name| {
        let name_string = &name.name;
        if is_valid_constant_name(name_string) {
            None
        } else {
            Some(InvalidItem::new(ValidatorKind::Constant, parsed, name.loc, name_string.clone()))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::utils::ExpectedFindings;

    #[test]
    fn test_validate() {
        let content = r"
            contract MyContract {
                // These have the constant or immutable keyword and should be valid.
                uint256 constant MAX_UINT256 = type(uint256).max;
                address constant ETH_ADDRESS = 0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE;

                // These have the constant/immutable keyword and should be invalid.
                bytes32 immutable zeroBytes = 0;
                int256 immutable minInt256 = type(int256).min;

                // These should all be valid since they are not constant or immutable.
                address alice = address(123);
                uint256 aliceBalance = 500;
            }
        ";

        let expected_findings = ExpectedFindings::new(2);
        expected_findings.assert_eq(content, &validate);
    }

    #[test]
    fn test_is_valid_constant_name() {
        let allowed_names = vec![
            "MAX_UINT256",
            "256_MAXUINT",
            "256_MAX_11_UINT",
            "VARIABLE",
            "VARIABLE_NAME",
            "VARIABLE_NAME_",
            "VARIABLE___NAME",
            "VARIABLE_NAME_WOW",
            "VARIABLE_NAME_WOW_AS_MANY_UNDERSCORES_AS_YOU_WANT",
            "__VARIABLE",
            "_VARIABLE__NAME",
            "_VARIABLE_NAME__",
            "_VARIABLE_NAME_WOW",
            "_VARIABLE_NAME_WOW_AS_MANY_UNDERSCORES_AS_YOU_WANT",
            "$VARIABLE_NAME",
            "_$VARIABLE_NAME_",
            "$_VARIABLE_NAME$",
            "_$VARIABLE_NAME$_",
            "$_VARIABLE_NAME_$",
            "$_VARIABLE__NAME_",
        ];

        let disallowed_names = [
            "variable",
            "variableName",
            "_variable",
            "_variable_Name",
            "VARIABLe",
            "VARIABLE_name",
            "_VARIABLe",
            "_VARIABLE_name",
            "$VARIABLe",
            "$VARIABLE_name",
        ];

        for name in allowed_names {
            assert!(is_valid_constant_name(name), "{name}");
        }

        for name in disallowed_names {
            assert!(!is_valid_constant_name(name), "{name}");
        }
    }
}
