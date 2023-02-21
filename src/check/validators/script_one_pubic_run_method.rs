use crate::check::utils::{
    FileKind, InvalidItem, IsFileKind, Name, ValidatorKind, VisibilitySummary,
};
use solang_parser::pt::{ContractPart, SourceUnit, SourceUnitPart};
use std::path::Path;

fn is_matching_file(file: &Path) -> bool {
    file.is_file_kind(FileKind::Script)
}

#[must_use]
/// Validates that a script has a single public method named `run`.
pub fn validate(file: &Path, _content: &str, pt: &SourceUnit) -> Vec<InvalidItem> {
    if !is_matching_file(file) {
        return Vec::new()
    }

    let mut public_methods: Vec<String> = Vec::new();
    for element in &pt.0 {
        if let SourceUnitPart::ContractDefinition(c) = element {
            for el in &c.parts {
                if let ContractPart::FunctionDefinition(f) = el {
                    let name = f.name();
                    if f.is_public_or_external() && name != "setUp" && name != "constructor" {
                        public_methods.push(name);
                    }
                }
            }
        }
    }

    // Parse the public methods found to return a vec that's either empty if valid, or has a single
    // invalid item otherwise.
    match public_methods.len() {
        0 => {
            vec![InvalidItem::new(
                ValidatorKind::Script,
                file.display().to_string(),
                "No `run` method found".to_string(),
                0, // This spans multiple lines, so we don't have a line number.
            )]
        }
        1 => {
            if public_methods[0] == "run" {
                Vec::new()
            } else {
                vec![InvalidItem::new(
                    ValidatorKind::Script,
                    file.display().to_string(),
                    "The only public method must be named `run`".to_string(),
                    0,
                )]
            }
        }
        _ => {
            vec![InvalidItem::new(
              ValidatorKind::Script,
              file.display().to_string(),
              format!("Scripts must have a single public method named `run` (excluding `setUp`), but the following methods were found: {public_methods:?}"),
              0,
          )]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::utils::ExpectedFindings;

    #[test]
    fn test_validate() {
        // TODO add another test for the third match arm
        let content_good = r#"
            contract MyContract {
                function run() public {}
            }
        "#;

        // The number after `bad` on the variable name indicates the match arm covered.
        let content_bad0 = r#"
            contract MyContract {}
        "#;

        let content_bad1 = r#"
            contract MyContract {
                function notRun() public {}
            }
        "#;

        let content_bad2_variant0 = r#"
            contract MyContract {
                function run() public {}
                function run(string memory config) public {}
            }
        "#;

        let content_bad2_variant1 = r#"
            contract MyContract {
                function run() public {}
                function foo() public {}
            }
        "#;

        let content_bad2_variant2 = r#"
            contract MyContract {
                function foo() public {}
                function bar() public {}
            }
        "#;

        let expected_findings_good = ExpectedFindings::new(0);
        expected_findings_good.assert_eq(content_good, &validate);

        let expected_findings_bad = ExpectedFindings { script: 1, ..Default::default() };
        expected_findings_bad.assert_eq(content_bad0, &validate);
        expected_findings_bad.assert_eq(content_bad1, &validate);
        expected_findings_bad.assert_eq(content_bad2_variant0, &validate);
        expected_findings_bad.assert_eq(content_bad2_variant1, &validate);
        expected_findings_bad.assert_eq(content_bad2_variant2, &validate);
    }
}
