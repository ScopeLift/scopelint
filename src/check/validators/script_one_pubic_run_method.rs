use crate::check::utils::{FileKind, InvalidItem, IsFileKind, Name, Validator, VisibilitySummary};
use solang_parser::pt::{ContractPart, SourceUnit, SourceUnitPart};
use std::{error::Error, path::Path};

pub fn validate(
    file: &Path,
    _content: &str,
    pt: &SourceUnit,
) -> Result<Vec<InvalidItem>, Box<dyn Error>> {
    if !file.is_file_kind(FileKind::ScriptContracts) {
        return Ok(Vec::new())
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
            Ok(vec![InvalidItem::new(
                Validator::Script,
                file.display().to_string(),
                "No `run` method found".to_string(),
                0, // This spans multiple lines, so we don't have a line number.
            )])
        }
        1 => {
            if public_methods[0] != "run" {
                Ok(vec![InvalidItem::new(
                    Validator::Script,
                    file.display().to_string(),
                    "The only public method must be named `run`".to_string(),
                    0,
                )])
            } else {
                Ok(Vec::new())
            }
        }
        _ => {
            Ok(vec![InvalidItem::new(
              Validator::Script,
              file.display().to_string(),
              format!("Scripts must have a single public method named `run` (excluding `setUp`), but the following methods were found: {public_methods:?}"),
              0,
          )])
        }
    }
}
