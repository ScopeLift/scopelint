use crate::check::utils::{offset_to_line, InvalidItem, Validator};
use once_cell::sync::Lazy;
use regex::Regex;
use solang_parser::pt::{
    ContractPart, SourceUnit, SourceUnitPart, VariableAttribute, VariableDefinition,
};
use std::{error::Error, path::Path};

// A regex matching valid constant names, see the `validate_constant_names_regex` test for examples.
static RE_VALID_CONSTANT_NAME: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:[$_]*[A-Z0-9][$_]*){1,}$").unwrap());

const fn is_matching_file(_file: &Path) -> bool {
    true
}

pub fn validate(
    file: &Path,
    content: &str,
    pt: &SourceUnit,
) -> Result<Vec<InvalidItem>, Box<dyn Error>> {
    if !is_matching_file(file) {
        return Ok(Vec::new())
    }

    let mut invalid_items: Vec<InvalidItem> = Vec::new();
    for element in &pt.0 {
        match element {
            SourceUnitPart::VariableDefinition(v) => {
                if let Some(invalid_item) = validate_name(file, content, v) {
                    invalid_items.push(invalid_item);
                }
            }
            SourceUnitPart::ContractDefinition(c) => {
                for el in &c.parts {
                    if let ContractPart::VariableDefinition(v) = el {
                        if let Some(invalid_item) = validate_name(file, content, v) {
                            invalid_items.push(invalid_item);
                        }
                    }
                }
            }
            _ => (),
        }
    }
    Ok(invalid_items)
}

fn is_valid_constant_name(name: &str) -> bool {
    RE_VALID_CONSTANT_NAME.is_match(name)
}

fn validate_name(file: &Path, content: &str, v: &VariableDefinition) -> Option<InvalidItem> {
    let is_constant = v
        .attrs
        .iter()
        .any(|a| matches!(a, VariableAttribute::Constant(_) | VariableAttribute::Immutable(_)));
    let name = &v.name.name;

    if is_constant && !is_valid_constant_name(name) {
        Some(InvalidItem::new(
            Validator::Constant,
            file.display().to_string(),
            name.clone(),
            offset_to_line(content, v.loc.start()),
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            assert_eq!(is_valid_constant_name(name), true, "{name}");
        }

        for name in disallowed_names {
            assert_eq!(is_valid_constant_name(name), false, "{name}");
        }
    }

    #[test]
    fn test_validate() {
        let content = r#"
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
        "#;

        let (pt, _comments) = solang_parser::parse(&content, 0).expect("Parsing failed");

        let invalid_items_script_helper =
            validate(Path::new("./script/MyContract.sol"), content, &pt).unwrap();
        let invalid_items_script =
            validate(Path::new("./script/MyContract.s.sol"), content, &pt).unwrap();
        let invalid_items_src = validate(Path::new("./src/MyContract.sol"), content, &pt).unwrap();
        let invalid_items_test_helper =
            validate(Path::new("./test/MyContract.sol"), content, &pt).unwrap();
        let invalid_items_test =
            validate(Path::new("./test/MyContract.t.sol"), content, &pt).unwrap();

        assert_eq!(invalid_items_script_helper.len(), 2);
        assert_eq!(invalid_items_script.len(), 2);
        assert_eq!(invalid_items_src.len(), 2);
        assert_eq!(invalid_items_test_helper.len(), 2);
        assert_eq!(invalid_items_test.len(), 2);
    }
}
