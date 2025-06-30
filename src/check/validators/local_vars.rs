use crate::check::{
    utils::{FileKind, InvalidItem, IsFileKind, ValidatorKind},
    Parsed,
};
use solang_parser::pt::{
    ContractPart, FunctionDefinition, SourceUnitPart, Statement, VariableDeclaration,
};
use std::path::Path;

fn is_matching_file(file: &Path) -> bool {
    file.is_file_kind(FileKind::Src)
}

#[must_use]
/// Validates that local variables (variables scoped to functions) are prefixed with an underscore.
pub fn validate(parsed: &Parsed) -> Vec<InvalidItem> {
    if !is_matching_file(&parsed.file) {
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
                    if let ContractPart::FunctionDefinition(f) = el {
                        invalid_items.extend(validate_function(parsed, f));
                    }
                }
            }
            _ => (),
        }
    }
    invalid_items
}

fn is_valid_local_var_name(name: &str) -> bool {
    name.starts_with('_')
}

fn validate_function(parsed: &Parsed, f: &FunctionDefinition) -> Vec<InvalidItem> {
    let mut invalid_items = Vec::new();

    // Check function parameters
    for (_, param) in &f.params {
        if let Some(p) = param {
            if let Some(name) = &p.name {
                if !is_valid_local_var_name(&name.name) {
                    invalid_items.push(InvalidItem::new(
                        ValidatorKind::Src,
                        parsed,
                        p.loc,
                        name.name.clone(),
                    ));
                }
            }
        }
    }

    // Check function returns (named returns)
    for (_, param) in &f.returns {
        if let Some(p) = param {
            if let Some(name) = &p.name {
                if !is_valid_local_var_name(&name.name) {
                    invalid_items.push(InvalidItem::new(
                        ValidatorKind::Src,
                        parsed,
                        p.loc,
                        name.name.clone(),
                    ));
                }
            }
        }
    }

    // Check function body
    if let Some(body) = &f.body {
        invalid_items.extend(validate_statement(parsed, body));
    }

    invalid_items
}

fn validate_statement(parsed: &Parsed, stmt: &Statement) -> Vec<InvalidItem> {
    let mut invalid_items = Vec::new();

    match stmt {
        Statement::VariableDefinition(loc, VariableDeclaration { name: Some(name), .. }, _) => {
            if !is_valid_local_var_name(&name.name) {
                invalid_items.push(InvalidItem::new(
                    ValidatorKind::Src,
                    parsed,
                    *loc,
                    name.name.clone(),
                ));
            }
        }
        Statement::VariableDefinition(_, _, _) => {}
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
        Statement::Try(_, _, returns, clauses) => {
            // Check try statement return parameters
            if let Some((params, stmt)) = returns {
                for (_, param) in params {
                    if let Some(p) = param {
                        if let Some(name) = &p.name {
                            if !is_valid_local_var_name(&name.name) {
                                invalid_items.push(InvalidItem::new(
                                    ValidatorKind::Src,
                                    parsed,
                                    p.loc,
                                    name.name.clone(),
                                ));
                            }
                        }
                    }
                }
                invalid_items.extend(validate_statement(parsed, stmt));
            }

            for clause in clauses {
                match clause {
                    solang_parser::pt::CatchClause::Simple(_, param, stmt) => {
                        if let Some(p) = param {
                            if let Some(name) = &p.name {
                                if !is_valid_local_var_name(&name.name) {
                                    invalid_items.push(InvalidItem::new(
                                        ValidatorKind::Src,
                                        parsed,
                                        p.loc,
                                        name.name.clone(),
                                    ));
                                }
                            }
                        }
                        invalid_items.extend(validate_statement(parsed, stmt));
                    }
                    solang_parser::pt::CatchClause::Named(_, _, param, stmt) => {
                        if let Some(name) = &param.name {
                            if !is_valid_local_var_name(&name.name) {
                                invalid_items.push(InvalidItem::new(
                                    ValidatorKind::Src,
                                    parsed,
                                    param.loc,
                                    name.name.clone(),
                                ));
                            }
                        }
                        invalid_items.extend(validate_statement(parsed, stmt));
                    }
                }
            }
        }
        Statement::Assembly { block, .. } => {
            invalid_items.extend(validate_yul_block(parsed, block));
        }
        _ => (),
    }

    invalid_items
}

fn validate_yul_block(parsed: &Parsed, block: &solang_parser::pt::YulBlock) -> Vec<InvalidItem> {
    let mut invalid_items = Vec::new();

    for stmt in &block.statements {
        match stmt {
            solang_parser::pt::YulStatement::VariableDeclaration(loc, names, _) => {
                for name in names {
                    if !is_valid_local_var_name(&name.id.name) {
                        invalid_items.push(InvalidItem::new(
                            ValidatorKind::Src,
                            parsed,
                            *loc,
                            name.id.name.clone(),
                        ));
                    }
                }
            }
            solang_parser::pt::YulStatement::Block(nested_block) => {
                invalid_items.extend(validate_yul_block(parsed, nested_block));
            }
            solang_parser::pt::YulStatement::If(_, _, block) => {
                invalid_items.extend(validate_yul_block(parsed, block));
            }
            solang_parser::pt::YulStatement::For(for_stmt) => {
                invalid_items.extend(validate_yul_block(parsed, &for_stmt.init_block));
                invalid_items.extend(validate_yul_block(parsed, &for_stmt.execution_block));
            }
            solang_parser::pt::YulStatement::Switch(switch) => {
                for case in &switch.cases {
                    match case {
                        solang_parser::pt::YulSwitchOptions::Case(_, _, block)
                        | solang_parser::pt::YulSwitchOptions::Default(_, block) => {
                            invalid_items.extend(validate_yul_block(parsed, block));
                        }
                    }
                }
            }
            _ => (),
        }
    }

    invalid_items
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::utils::ExpectedFindings;

    #[test]
    fn test_validate() {
        let content = r"
            contract MyContract {
                uint256 public someState;

                // Valid local variables (prefixed with underscore)
                function validFunction(uint256 _param1, address _param2) public returns (uint256 _result) {
                    uint256 _localVar = 42;
                    address _localAddr = address(0);
                    
                    for (uint256 _i = 0; _i < 10; _i++) {
                        uint256 _temp = _i * 2;
                    }
                    
                    return _localVar;
                }

                // Invalid local variables (not prefixed with underscore)
                function invalidFunction(uint256 param1, address param2) public returns (uint256 result) {
                    uint256 localVar = 42;
                    address localAddr = address(0);
                    
                    for (uint256 i = 0; i < 10; i++) {
                        uint256 temp = i * 2;
                    }
                    
                    if (localVar > 0) {
                        uint256 anotherVar = localVar * 2;
                    }
                    
                    return localVar;
                }

                // Test assembly blocks
                function assemblyFunction() public {
                    assembly {
                        let validVar := 1  // Should be invalid (no underscore)
                        let _validVar2 := 2  // Should be valid
                    }
                }

                // Test try/catch
                function tryCatchFunction() public {
                    try someContract.foo() returns (uint256 value) {  // value should be invalid
                        uint256 _temp = value;
                    } catch Error(string memory reason) {  // reason should be invalid
                        uint256 errorCase = 1;  // errorCase should be invalid
                    }
                }
            }
        ";

        // Count expected violations:
        // invalidFunction: 2 params + 1 return + 5 local vars = 8
        // assemblyFunction: 1 assembly var = 1
        // tryCatchFunction: 1 return + 1 catch param + 1 local var = 3
        // Total: 8 + 1 + 3 = 12
        let expected_findings = ExpectedFindings { src: 12, ..ExpectedFindings::default() };
        expected_findings.assert_eq(content, &validate);
    }

    #[test]
    fn test_nested_statements() {
        let content = r"
            contract NestedTest {
                function nested() public {
                    uint256 _valid = 1;
                    
                    if (true) {
                        uint256 invalid1 = 2;  // invalid
                        
                        while (_valid > 0) {
                            uint256 invalid2 = 3;  // invalid
                            
                            do {
                                uint256 _valid2 = 4;
                                uint256 invalid3 = 5;  // invalid
                            } while (_valid2 > 0);
                        }
                    } else {
                        uint256 invalid4 = 6;  // invalid
                    }
                }
            }
        ";

        let expected_findings = ExpectedFindings { src: 4, ..ExpectedFindings::default() };
        expected_findings.assert_eq(content, &validate);
    }
}
