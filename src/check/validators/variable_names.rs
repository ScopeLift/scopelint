use crate::check::{
    utils::{FileKind, InvalidItem, IsFileKind, ValidatorKind},
    Parsed,
};
use solang_parser::pt::{
    ContractPart, FunctionDefinition, Parameter, SourceUnitPart, Statement, VariableDeclaration,
    VariableDefinition,
};
fn is_matching_file(parsed: &Parsed) -> bool {
    let file = &parsed.file;
    file.is_file_kind(FileKind::Src, &parsed.path_config) ||
        file.is_file_kind(FileKind::Test, &parsed.path_config) ||
        file.is_file_kind(FileKind::Handler, &parsed.path_config) ||
        file.is_file_kind(FileKind::Script, &parsed.path_config)
}

#[must_use]
/// Validates that variable names follow the correct naming conventions:
/// - Storage variables should NOT have an underscore prefix
/// - Non-storage variables (local variables, parameters) should have an underscore prefix
/// - Variables that reference storage/storages should NOT have an underscore prefix
pub fn validate(parsed: &Parsed) -> Vec<InvalidItem> {
    if !is_matching_file(parsed) {
        return Vec::new();
    }

    let mut invalid_items: Vec<InvalidItem> = Vec::new();
    for element in &parsed.pt.0 {
        match element {
            SourceUnitPart::FunctionDefinition(f) => {
                invalid_items.extend(validate_function(parsed, f));
            }
            SourceUnitPart::ContractDefinition(c) => {
                for el in &c.parts {
                    match el {
                        ContractPart::FunctionDefinition(f) => {
                            invalid_items.extend(validate_function(parsed, f));
                        }
                        ContractPart::VariableDefinition(v) => {
                            if let Some(invalid_item) = validate_state_variable(parsed, v) {
                                invalid_items.push(invalid_item);
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => (),
        }
    }
    invalid_items
}

fn validate_function(parsed: &Parsed, f: &FunctionDefinition) -> Vec<InvalidItem> {
    let mut invalid_items: Vec<InvalidItem> = Vec::new();

    // Validate function parameters
    for (_, param) in &f.params {
        if let Some(p) = param {
            if let Some(name) = &p.name {
                let is_storage = is_storage_parameter(p);
                if !is_valid_parameter_name(&name.name, is_storage) {
                    let message = if is_storage {
                        format!(
                            "Storage parameter '{}' should NOT have underscore prefix",
                            &name.name
                        )
                    } else {
                        format!("Parameter '{}' should have underscore prefix", &name.name)
                    };
                    invalid_items.push(InvalidItem::new(
                        ValidatorKind::Variable,
                        parsed,
                        p.loc,
                        message,
                    ));
                }
            }
        }
    }

    // Validate local variables in function body
    if let Some(body) = &f.body {
        invalid_items.extend(validate_statement(parsed, body));
    }

    invalid_items
}

fn validate_state_variable(parsed: &Parsed, v: &VariableDefinition) -> Option<InvalidItem> {
    v.name.as_ref().and_then(|name| {
        let name_str = &name.name;
        if is_valid_state_variable_name(name_str) {
            None
        } else {
            Some(InvalidItem::new(
                ValidatorKind::Variable,
                parsed,
                name.loc,
                format!("State variable '{name_str}' should NOT have underscore prefix"),
            ))
        }
    })
}

fn validate_statement(parsed: &Parsed, stmt: &Statement) -> Vec<InvalidItem> {
    let mut invalid_items = Vec::new();

    match stmt {
        Statement::VariableDefinition(
            loc,
            VariableDeclaration { name: Some(name), storage, .. },
            _,
        ) => {
            // Check if this is a storage variable by examining the storage location
            let is_storage =
                matches!(storage, Some(solang_parser::pt::StorageLocation::Storage(_)));

            if !is_valid_local_variable_name(&name.name, is_storage) {
                let message = if is_storage {
                    format!("Storage variable '{}' should NOT have underscore prefix", &name.name)
                } else {
                    format!("Local variable '{}' should have underscore prefix", &name.name)
                };
                invalid_items.push(InvalidItem::new(
                    ValidatorKind::Variable,
                    parsed,
                    *loc,
                    message,
                ));
            }
        }
        Statement::Block { statements, .. } => {
            for s in statements {
                invalid_items.extend(validate_statement(parsed, s));
            }
        }
        Statement::If(_, _, then_stmt, else_stmt) => {
            invalid_items.extend(validate_statement(parsed, then_stmt));
            if let Some(else_s) = else_stmt {
                invalid_items.extend(validate_statement(parsed, else_s));
            }
        }
        Statement::While(_, _, body) | Statement::DoWhile(_, body, _) => {
            invalid_items.extend(validate_statement(parsed, body));
        }
        Statement::For(_, init, _, _, body) => {
            if let Some(init_stmt) = init {
                invalid_items.extend(validate_statement(parsed, init_stmt));
            }
            if let Some(body_stmt) = body {
                invalid_items.extend(validate_statement(parsed, body_stmt));
            }
        }
        _ => {}
    }

    invalid_items
}

const fn is_storage_parameter(param: &Parameter) -> bool {
    // Check if the parameter has storage location set to Storage
    // This is the proper way to detect storage parameters
    if let Some(storage_location) = &param.storage {
        matches!(storage_location, solang_parser::pt::StorageLocation::Storage(_))
    } else {
        false
    }
}

fn is_valid_parameter_name(name: &str, is_storage: bool) -> bool {
    if is_storage {
        // Storage parameters should NOT have underscore prefix
        !name.starts_with('_')
    } else {
        // Non-storage parameters should have underscore prefix
        name.starts_with('_')
    }
}

fn is_valid_state_variable_name(name: &str) -> bool {
    // State variables should NOT have underscore prefix
    !name.starts_with('_')
}

fn is_valid_local_variable_name(name: &str, is_storage: bool) -> bool {
    if is_storage {
        // Storage variables should NOT have underscore prefix
        !name.starts_with('_')
    } else {
        // Non-storage variables should have underscore prefix
        name.starts_with('_')
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::utils::ExpectedFindings;

    #[test]
    fn test_all_valid_variable_names() {
        let content = r"
            contract MyContract {
                uint256 validStateVar;
                uint256 constant VALID_CONSTANT = 123;
                uint256 immutable validImmutable = 456;
                
                function validFunction(uint256 _param1, address _param2) external {
                    uint256 _localVar = 123;
                    address _user = msg.sender;
                    Deposit storage deposit = deposits[_param1];
                }

                function validStorageFunction(Deposit storage deposit) external {
                    // Function body
                }
            }
        ";

        let expected_findings = ExpectedFindings::new(0);
        expected_findings.assert_eq(content, &validate);
    }

    #[test]
    fn test_state_variable_with_underscore() {
        let content = r"
            contract MyContract {
                uint256 _invalidStateVar;
            }
        ";

        let expected_findings = ExpectedFindings {
            src: 1,
            test: 1,
            handler: 1,
            script: 1,
            ..ExpectedFindings::default()
        };
        expected_findings.assert_eq(content, &validate);
    }

    #[test]
    fn test_storage_parameter_with_underscore() {
        let content = r"
            contract MyContract {
                function invalidStorageFunction(Deposit storage _deposit) external {
                    // Function body
                }
            }
        ";

        let expected_findings = ExpectedFindings {
            src: 1,
            test: 1,
            handler: 1,
            script: 1,
            ..ExpectedFindings::default()
        };
        expected_findings.assert_eq(content, &validate);
    }

    #[test]
    fn test_non_storage_parameter_without_underscore() {
        let content = r"
            contract MyContract {
                function invalidFunction(uint256 param1, address param2) external {
                    // Function body
                }
            }
        ";

        let expected_findings = ExpectedFindings {
            src: 2,
            test: 2,
            handler: 2,
            script: 2,
            ..ExpectedFindings::default()
        };
        expected_findings.assert_eq(content, &validate);
    }

    #[test]
    fn test_local_variable_without_underscore() {
        let content = r"
            contract MyContract {
                function invalidFunction() external {
                    uint256 localVar = 123;
                    address user = msg.sender;
                }
            }
        ";

        let expected_findings = ExpectedFindings {
            src: 2,
            test: 2,
            handler: 2,
            script: 2,
            ..ExpectedFindings::default()
        };
        expected_findings.assert_eq(content, &validate);
    }

    #[test]
    fn test_storage_variable_with_underscore() {
        let content = r"
            contract MyContract {
                function invalidFunction() external {
                    Deposit storage _deposit = deposits[0];
                }
            }
        ";

        let expected_findings = ExpectedFindings {
            src: 1,
            test: 1,
            handler: 1,
            script: 1,
            ..ExpectedFindings::default()
        };
        expected_findings.assert_eq(content, &validate);
    }
}
