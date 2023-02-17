use crate::check::utils::{
    offset_to_line, FileKind, InvalidItem, IsFileKind, Name, Validator, VisibilitySummary,
};
use solang_parser::pt::{ContractPart, FunctionDefinition, SourceUnit, SourceUnitPart};
use std::{error::Error, path::Path};

pub fn validate(
    file: &Path,
    content: &str,
    pt: &SourceUnit,
) -> Result<Vec<InvalidItem>, Box<dyn Error>> {
    if !file.is_file_kind(FileKind::SrcContracts) {
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
            Validator::Src,
            file.display().to_string(),
            name,
            offset_to_line(content, f.loc.start()),
        ))
    } else {
        None
    }
}
