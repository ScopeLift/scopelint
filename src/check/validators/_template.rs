use crate::check::{
    utils::{offset_to_line, InvalidItem, Validator},
    Parsed,
};
use solang_parser::pt::{
    ContractPart, SourceUnit, SourceUnitPart, VariableAttribute, VariableDefinition,
};
use std::path::Path;

const fn is_matching_file(_file: &Path) -> bool {
    true // Update the matching condition, see helpers in `src/check/utils.rs`.
}

#[must_use]
/// Validates that <explain validator>.
pub fn validate(parsed: &Parsed) -> Vec<InvalidItem> {
    let Parsed { file, src, pt, .. } = parsed;
    if !is_matching_file(file) {
        return Vec::new()
    }

    let mut invalid_items: Vec<InvalidItem> = Vec::new();
    // Edit below here to add your own validation logic.
    for element in &pt.0 {
        match element {
            _ => (),
        }
    }
    invalid_items
}

// Add any helper methods here. The `validate` method should be the only public method.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::utils::ExpectedFindings;

    #[test]
    fn test_validate() {
        let content = r#"
            contract MyContract {
                // Fill in one or more sample contracts. See `script_one_pubic_run_method.rs` for
                // an example of testing more contracts.
            }
        "#;

        let expected_findings = ExpectedFindings::new(0);
        expected_findings.assert_eq(content, &validate);
    }

    // Add any other unit tests here. Tests are named `test_<function_name>`. See
    // `constant_names.rs` for an example of additional unit tests.
}
