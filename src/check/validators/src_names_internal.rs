use crate::check::{
    utils::{FileKind, InvalidItem, IsFileKind, Name, ValidatorKind, VisibilitySummary},
    Parsed,
};
use solang_parser::pt::{ContractPart, ContractTy, FunctionDefinition, SourceUnitPart};
use std::path::Path;

fn is_matching_file(file: &Path) -> bool {
    file.is_file_kind(FileKind::Src)
}

#[must_use]
/// Validates that internal and private function names are prefixed with an underscore.
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
                if !matches!(c.ty, ContractTy::Library(_)) {
                    for el in &c.parts {
                        if let ContractPart::FunctionDefinition(f) = el {
                            if let Some(invalid_item) = validate_name(parsed, f) {
                                invalid_items.push(invalid_item);
                            }
                        }
                    }
                }
            }
            _ => (),
        }
    }
    invalid_items
}

fn is_valid_internal_or_private_name(name: &str) -> bool {
    name.starts_with('_')
}

fn validate_name(parsed: &Parsed, f: &FunctionDefinition) -> Option<InvalidItem> {
    let name = f.name();
    if f.is_internal_or_private() && !is_valid_internal_or_private_name(&name) {
        Some(InvalidItem::new(ValidatorKind::Src, parsed, f.name_loc, name))
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
                // Valid names for internal or private src methods.
                function _myInternalMethod() internal {}
                function _myPrivateMethod() private {}

                // Invalid names for internal or private src methods.
                function myInternalMethod() internal {}
                function myPrivateMethod() private {}

                // These should be ignored since they are public and external.
                function myPublicMethod() public {}
                function myExternalMethod() external {}
            }
        ";

        let expected_findings = ExpectedFindings { src: 2, ..ExpectedFindings::default() };
        expected_findings.assert_eq(content, &validate);
    }
}
