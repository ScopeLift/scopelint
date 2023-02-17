use crate::check::{
    report::{InvalidItem, Validator},
    utils::{offset_to_line, FileKind, IsFileKind, Name, VisibilitySummary},
};
use solang_parser::pt::{ContractPart, SourceUnit, SourceUnitPart};
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
                let name = f.name();
                if f.is_internal_or_private() && !is_valid_internal_or_private_name(&name) {
                    invalid_items.push(InvalidItem::new(
                        Validator::Src,
                        file.display().to_string(),
                        name.to_string(),
                        offset_to_line(content, f.loc.start()),
                    ));
                }
            }
            SourceUnitPart::ContractDefinition(c) => {
                for el in &c.parts {
                    if let ContractPart::FunctionDefinition(f) = el {
                        let name = f.name();
                        if f.is_internal_or_private() && !is_valid_internal_or_private_name(&name) {
                            invalid_items.push(InvalidItem::new(
                                Validator::Src,
                                file.display().to_string(),
                                name.to_string(),
                                offset_to_line(content, f.loc.start()),
                            ));
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
