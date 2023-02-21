use crate::check::utils::{FileKind, InvalidItem, IsFileKind, Name, Validator, VisibilitySummary};
use solang_parser::pt::{ContractPart, SourceUnit, SourceUnitPart};
use std::{error::Error, path::Path};

fn is_matching_file(file: &Path) -> bool {
    file.is_file_kind(FileKind::ScriptContracts)
}

pub fn validate(
    file: &Path,
    _content: &str,
    pt: &SourceUnit,
) -> Result<Vec<InvalidItem>, Box<dyn Error>> {
    if !is_matching_file(file) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate() {
        // TODO add another test for the third match arm
        let content_good = r#"
            contract MyContract {
                function run() public {}
            }
        "#;

        let content_bad = r#"
            contract MyContract {
                function run() public {}
                function run(string memory config) public {}
            }
        "#;

        let (pt_good, _comments) = solang_parser::parse(&content_good, 0).expect("Parsing failed");
        let (pt_bad, _comments) = solang_parser::parse(&content_bad, 0).expect("Parsing failed");

        let invalid_items_script_helper_good =
            validate(Path::new("./script/MyContract.sol"), content_good, &pt_good).unwrap();
        let invalid_items_script_good =
            validate(Path::new("./script/MyContract.s.sol"), content_good, &pt_good).unwrap();
        let invalid_items_src_good =
            validate(Path::new("./src/MyContract.sol"), content_good, &pt_good).unwrap();
        let invalid_items_test_helper_good =
            validate(Path::new("./test/MyContract.sol"), content_good, &pt_good).unwrap();
        let invalid_items_test_good =
            validate(Path::new("./test/MyContract.t.sol"), content_good, &pt_good).unwrap();

        let invalid_items_script_helper_bad =
            validate(Path::new("./script/MyContract.sol"), content_bad, &pt_bad).unwrap();
        let invalid_items_script_bad =
            validate(Path::new("./script/MyContract.s.sol"), content_bad, &pt_bad).unwrap();
        let invalid_items_src_bad =
            validate(Path::new("./src/MyContract.sol"), content_bad, &pt_bad).unwrap();
        let invalid_items_test_helper_bad =
            validate(Path::new("./test/MyContract.sol"), content_bad, &pt_bad).unwrap();
        let invalid_items_test_bad =
            validate(Path::new("./test/MyContract.t.sol"), content_bad, &pt_bad).unwrap();

        assert_eq!(invalid_items_script_helper_good.len(), 0);
        assert_eq!(invalid_items_script_good.len(), 0);
        assert_eq!(invalid_items_src_good.len(), 0);
        assert_eq!(invalid_items_test_helper_good.len(), 0);
        assert_eq!(invalid_items_test_good.len(), 0);

        assert_eq!(invalid_items_script_helper_bad.len(), 0);
        assert_eq!(invalid_items_script_bad.len(), 1);
        assert_eq!(invalid_items_src_bad.len(), 0);
        assert_eq!(invalid_items_test_helper_bad.len(), 0);
        assert_eq!(invalid_items_test_bad.len(), 0);
    }
}
