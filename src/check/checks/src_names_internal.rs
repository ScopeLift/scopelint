use crate::check::{
    report::{InvalidItem, Validator},
    utils::{offset_to_line, FileKind, IsFileKind, Name},
};
use solang_parser::pt::{
    ContractPart, FunctionAttribute, FunctionDefinition, SourceUnit, SourceUnitPart, Visibility,
};
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
                if is_private(f) && !is_valid_internal_or_private_name(&name) {
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
                        if is_private(f) && !is_valid_internal_or_private_name(&name) {
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

fn is_private(func_def: &FunctionDefinition) -> bool {
    func_def.attributes.iter().any(|a| match a {
        FunctionAttribute::Visibility(v) => {
            matches!(v, Visibility::Private(_) | Visibility::Internal(_))
        }
        _ => false,
    })
}
