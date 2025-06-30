use crate::check::{
    utils::{FileKind, InvalidItem, IsFileKind, Name, ValidatorKind, VisibilitySummary},
    Parsed,
};
use solang_parser::pt::{ContractPart, Loc, SourceUnitPart};
use std::path::Path;

fn is_matching_file(file: &Path) -> bool {
    file.is_file_kind(FileKind::Script)
}

#[must_use]
/// Validates that a script has a single public method named `run`.
///
/// # Panics
///
/// Panics if the script has no contract definition.
pub fn validate(parsed: &Parsed) -> Vec<InvalidItem> {
    if !is_matching_file(&parsed.file) {
        return Vec::new();
    }

    // The location of findings spans multiple lines, so we use the contract location.
    let mut contract_loc: Option<Loc> = None;

    // Find all public methods that aren't `setUp` or `constructor`.
    let mut public_methods: Vec<String> = Vec::new();
    for element in &parsed.pt.0 {
        if let SourceUnitPart::ContractDefinition(c) = element {
            contract_loc = Some(c.loc);
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
                parsed,
                contract_loc.unwrap(), //
                "No `run` method found".to_string(),
            )]
        }
        _ => {
            if public_methods.contains(&"run".to_string()) {
                Vec::new()
            } else {
                vec![InvalidItem::new(
                    ValidatorKind::Script,
                    parsed,
                    contract_loc.unwrap(), //
                    "No `run` method found".to_string(),
                )]
            }
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
        let content_good = r"
            contract MyContract {
                function run() public {}
            }
        ";

        let content_good_variant0 = r"
            contract MyContract {
                function run() public {}
                function run(string memory config) public {}
            }
        ";

        let content_good_variant1 = r"
            contract MyContract {
                function run() public {}
                function foo() public {}
            }
        ";

        let content_good_variant2 = r"
            contract MyContract {
                function run(address admin) public {}
            }
        ";

        // The number after `bad` on the variable name indicates the match arm covered.
        let content_bad0 = r"
            contract MyContract {}
        ";

        let content_bad1 = r"
            contract MyContract {
                function notRun() public {}
            }
        ";

        let content_bad2_variant0 = r"
            contract MyContract {
                function foo() public {}
                function bar() public {}
            }
        ";

        let expected_findings_good = ExpectedFindings::new(0);
        expected_findings_good.assert_eq(content_good, &validate);
        expected_findings_good.assert_eq(content_good_variant0, &validate);
        expected_findings_good.assert_eq(content_good_variant1, &validate);
        expected_findings_good.assert_eq(content_good_variant2, &validate);

        let expected_findings_bad = ExpectedFindings { script: 1, ..Default::default() };
        expected_findings_bad.assert_eq(content_bad0, &validate);
        expected_findings_bad.assert_eq(content_bad1, &validate);
        expected_findings_bad.assert_eq(content_bad2_variant0, &validate);
    }
}
