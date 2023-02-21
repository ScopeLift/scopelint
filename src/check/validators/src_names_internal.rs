use crate::check::utils::{
    offset_to_line, FileKind, InvalidItem, IsFileKind, Name, ValidatorKind, VisibilitySummary,
};
use solang_parser::pt::{ContractPart, FunctionDefinition, SourceUnit, SourceUnitPart};
use std::{error::Error, path::Path};

fn is_matching_file(file: &Path) -> bool {
    file.is_file_kind(FileKind::SrcContracts)
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
            SourceUnitPart::FunctionDefinition(f) => {
                if let Some(invalid_item) = validate_name(file, content, f) {
                    invalid_items.push(invalid_item);
                }
            }
            SourceUnitPart::ContractDefinition(c) => {
                for el in &c.parts {
                    if let ContractPart::FunctionDefinition(f) = el {
                        if let Some(invalid_item) = validate_name(file, content, f) {
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

fn is_valid_internal_or_private_name(name: &str) -> bool {
    name.starts_with('_')
}

fn validate_name(file: &Path, content: &str, f: &FunctionDefinition) -> Option<InvalidItem> {
    let name = f.name();
    if f.is_internal_or_private() && !is_valid_internal_or_private_name(&name) {
        Some(InvalidItem::new(
            ValidatorKind::Src,
            file.display().to_string(),
            name,
            offset_to_line(content, f.loc.start()),
        ))
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
        let content = r#"
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
        "#;

        let expected_findings = ExpectedFindings { src: 2, ..ExpectedFindings::default() };
        expected_findings.assert_eq(content, &validate);
    }
}
