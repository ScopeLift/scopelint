use regex::Regex;
use solang_parser::pt::{ContractPart, SourceUnitPart, VariableDefinition};

use crate::check::{
    utils::{FileKind, InvalidItem, IsFileKind, ValidatorKind},
    Parsed,
};
use std::path::Path;

#[must_use]
// Validates that EIP712 typehash parameter counts match their usage in abi.encode calls.
pub fn validate(parsed: &Parsed) -> Vec<InvalidItem> {
    if !is_matching_file(&parsed.file) {
        return Vec::new();
    }

    let mut invalid_items: Vec<InvalidItem> = Vec::new();
    let mut typehash_variables: Vec<(String, String, solang_parser::pt::Loc, Option<String>)> =
        Vec::new();

    // Collect typehash variables from contracts
    for element in &parsed.pt.0 {
        if let SourceUnitPart::ContractDefinition(c) = element {
            for el in &c.parts {
                if let ContractPart::VariableDefinition(v) = el {
                    if let Some(typehash_info) = extract_typehash_variable(v) {
                        typehash_variables.push(typehash_info);
                    }
                }
            }
        }
    }

    // Validate typehashes - extract parameter count and compare with usage
    for (typehash_name, expected_struct_name, loc, keccak_string) in typehash_variables {
        if let Some(keccak_content) = &keccak_string {
            // Extract parameter count from keccak256 string
            // Example: "Permit(address owner,address spender,uint256 value,uint256 nonce,uint256
            // deadline)" -> 5 parameters
            let param_count = extract_parameter_count(keccak_content);

            // Find all usages of this typehash and check each one
            let usages = find_all_typehash_usages(parsed, &typehash_name);

            for usage_param_count in usages {
                if usage_param_count != param_count {
                    invalid_items.push(InvalidItem::new(
                        ValidatorKind::Eip712,
                        parsed,
                        loc,
                        format!("EIP712 typehash '{typehash_name}' parameter mismatch: typehash defines {param_count} parameters but abi.encode usage uses {usage_param_count} parameters"),
                    ));
                }
            }
        } else {
            // No keccak256 string found - this is definitely an issue
            invalid_items.push(InvalidItem::new(
                ValidatorKind::Eip712,
                parsed,
                loc,
                format!("Typehash '{typehash_name}' for struct '{expected_struct_name}' has no keccak256 string - this will cause signature mismatches"),
            ));
        }
    }

    invalid_items
}

fn is_matching_file(file: &Path) -> bool {
    file.is_file_kind(FileKind::Src)
}

fn extract_typehash_variable(
    v: &VariableDefinition,
) -> Option<(String, String, solang_parser::pt::Loc, Option<String>)> {
    // Must have TYPEHASH in the name
    let var_name = v.name.as_ref()?;
    let name = &var_name.name;

    // Check if it's a typehash variable
    if !name.ends_with("_TYPEHASH") && !name.starts_with("TYPEHASH_") {
        return None;
    }

    // Extract struct name and keccak256 string
    let struct_name = if name.ends_with("_TYPEHASH") {
        name.strip_suffix("_TYPEHASH").unwrap_or(name)
    } else {
        name.strip_prefix("TYPEHASH_").unwrap_or(name)
    };

    let keccak_string = extract_keccak256_string(v);
    Some((name.clone(), struct_name.to_string(), var_name.loc, keccak_string))
}

fn extract_keccak256_string(v: &VariableDefinition) -> Option<String> {
    if let Some(initializer) = &v.initializer {
        let source_snippet = format!("{initializer:?}");

        // Extract string from StringLiteral structure
        let re = Regex::new(r#"string:\s*"([^"]+)"#).ok()?;
        if let Some(captures) = re.captures(&source_snippet) {
            if let Some(string_content) = captures.get(1) {
                return Some(string_content.as_str().to_string());
            }
        }
    }

    None
}

// Extract parameter count from keccak256 string
fn extract_parameter_count(keccak_string: &str) -> usize {
    // Example: "Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)"
    // Extract the part between parentheses and count the parameters
    let re = Regex::new(r"\(([^)]+)\)").ok();
    if let Some(regex) = re {
        if let Some(captures) = regex.captures(keccak_string) {
            if let Some(params_str) = captures.get(1) {
                // Split by comma and count
                return params_str.as_str().split(',').count();
            }
        }
    }
    0
}

// Find all usages of a typehash and return parameter counts
fn find_all_typehash_usages(parsed: &Parsed, typehash_name: &str) -> Vec<usize> {
    let source = &parsed.src;
    let mut usages = Vec::new();

    // Look for abi.encode patterns with the typehash and capture the parameters
    let pattern = format!(r"abi\.encode\s*\(\s*{typehash_name}\s*,\s*([^)]+)\)");
    // Create regex to find abi.encode calls with our typehash
    if let Ok(regex) = Regex::new(&pattern) {
        // Find all matches in the source code
        for captures in regex.captures_iter(source) {
            // Extract the parameters part (captured group 1)
            if let Some(param_group) = captures.get(1) {
                let parameters_text = param_group.as_str();

                // Count parameters: "a, b, c" has 2 commas = 3 parameters
                let param_count = parameters_text.matches(',').count() + 1;

                usages.push(param_count);
            }
        }
    }

    usages
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::utils::ExpectedFindings;

    #[test]
    fn test_validate() {
        let content = r"
            contract MyContract {
                // Typehash constant that is used correctly (should not flag)
                bytes32 constant STAKE_TYPEHASH = keccak256('Stake(uint256 amount,address delegatee,address claimer,address depositor,uint256 nonce,uint256 deadline)');
                
                function stakeOnBehalf(uint256 amount, address delegatee, address claimer, address depositor, uint256 deadline, bytes memory signature) external {
                    // Correct usage - 6 parameters match the typehash definition
                    bytes32 hash = keccak256(abi.encode(STAKE_TYPEHASH, amount, delegatee, claimer, depositor, nonce, deadline));
                }
                
                // Typehash constant that is used incorrectly (should flag)
                bytes32 constant WRONG_TYPEHASH = keccak256('Wrong(uint256 param1,uint256 param2,uint256 param3)');
                
                function wrongUsage() external {
                    // Wrong usage - 3 parameters defined but only 2 used
                    bytes32 hash = keccak256(abi.encode(WRONG_TYPEHASH, param1, param2));
                }
            }
        ";

        let expected_findings = ExpectedFindings { src: 1, test: 0, ..ExpectedFindings::default() };
        expected_findings.assert_eq(content, &validate);
    }

    #[test]
    fn test_abi_encode_packed() {
        let content = r"
            contract MyContract {
                bytes32 constant PERMIT_TYPEHASH = keccak256('Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)');
                
                function permit() external {
                    // Should NOT flag - abi.encodePacked is not supported in this simplified version
                    bytes32 hash = keccak256(abi.encodePacked(PERMIT_TYPEHASH, owner, spender, value));
                }
            }
        ";

        let expected_findings = ExpectedFindings { src: 0, test: 0, ..ExpectedFindings::default() };
        expected_findings.assert_eq(content, &validate);
    }

    #[test]
    fn test_nested_keccak256() {
        let content = r"
            contract MyContract {
                bytes32 constant WITHDRAW_TYPEHASH = keccak256('Withdraw(uint256 depositId,uint256 amount,address depositor,uint256 nonce,uint256 deadline)');
                
                function withdraw() external {
                    // Should flag - 5 parameters defined but only 2 used
                    bytes32 hash = keccak256(abi.encode(WITHDRAW_TYPEHASH, depositId, amount));
                }
            }
        ";

        let expected_findings = ExpectedFindings { src: 1, test: 0, ..ExpectedFindings::default() };
        expected_findings.assert_eq(content, &validate);
    }

    #[test]
    fn test_multiple_usages() {
        let content = r"
            contract MyContract {
                bytes32 constant CLAIM_TYPEHASH = keccak256('Claim(uint256 depositId,uint256 nonce,uint256 deadline)');
                
                function claim1() external {
                    // Correct usage - 3 parameters
                    bytes32 hash = keccak256(abi.encode(CLAIM_TYPEHASH, depositId, nonce, deadline));
                }
                
                function claim2() external {
                    // Wrong usage - 3 parameters defined but only 2 used
                    bytes32 hash = keccak256(abi.encode(CLAIM_TYPEHASH, depositId, nonce));
                }
            }
        ";

        let expected_findings = ExpectedFindings { src: 1, test: 0, ..ExpectedFindings::default() };
        expected_findings.assert_eq(content, &validate);
    }

    #[test]
    fn test_unused_typehash() {
        let content = r"
            contract MyContract {
                // Should NOT flag - unused typehash
                bytes32 constant UNUSED_TYPEHASH = keccak256('Unused(uint256 param1,uint256 param2)');
                
                // Should NOT flag - no typehash constants
                function normalFunction() external {
                    bytes32 hash = keccak256('Normal(string message)');
                }
            }
        ";

        let expected_findings = ExpectedFindings { src: 0, test: 0, ..ExpectedFindings::default() };
        expected_findings.assert_eq(content, &validate);
    }

    #[test]
    fn test_non_constant_typehash() {
        let content = r"
            contract MyContract {
                // Should detect non-constant typehash
                bytes32 private PERMIT_TYPEHASH = keccak256('Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)');
                
                function permit() external {
                    // Should flag - 5 parameters defined but only 3 used
                    bytes32 hash = keccak256(abi.encode(PERMIT_TYPEHASH, owner, spender, value));
                }
            }
        ";

        let expected_findings = ExpectedFindings { src: 1, test: 0, ..ExpectedFindings::default() };
        expected_findings.assert_eq(content, &validate);
    }

    #[test]
    fn test_invalid_typehash_initializer() {
        let content = r"
            contract MyContract {
                // Should flag - invalid typehash initializer (function call instead of string literal)
                bytes32 constant PERMIT_TYPEHASH = keccak256(someFunction());
                
                function permit() external {
                    bytes32 hash = keccak256(abi.encode(PERMIT_TYPEHASH, owner, spender, value));
                }
            }
        ";

        let expected_findings = ExpectedFindings { src: 1, test: 0, ..ExpectedFindings::default() };
        expected_findings.assert_eq(content, &validate);
    }

    #[test]
    fn test_tuple_parameters() {
        let content = r"
            contract MyContract {
                // Typehash with tuple parameters
                bytes32 constant COMPLEX_TYPEHASH = keccak256('Complex(address owner,(uint256 amount,address token)[] positions,uint256 deadline)');
                
                function complexOperation() external {
                    // Should flag - parameter count mismatch
                    // Typehash defines: owner, positions[], deadline (3 parameters)
                    // But usage has: owner, positions, deadline, extraParam (4 parameters)
                    bytes32 hash = keccak256(abi.encode(COMPLEX_TYPEHASH, owner, positions, deadline, extraParam));
                }
                
                function correctComplexOperation() external {
                    // Should NOT flag - correct usage with 3 parameters
                    bytes32 hash = keccak256(abi.encode(COMPLEX_TYPEHASH, owner, positions, deadline));
                }
            }
        ";

        let expected_findings = ExpectedFindings { src: 1, test: 0, ..ExpectedFindings::default() };
        expected_findings.assert_eq(content, &validate);
    }

    #[test]
    fn test_nested_tuple_parameters() {
        let content = r"
            contract MyContract {
                // Typehash with nested tuple parameters
                bytes32 constant NESTED_TYPEHASH = keccak256('Nested(address user,(uint256 id,(string name,uint256 value)[] items)[] batches,uint256 timestamp)');
                
                function nestedOperation() external {
                    // Should flag - parameter count mismatch
                    // Typehash defines: user, batches[], timestamp (3 parameters)
                    // But usage has: user, batches, timestamp, extraParam (4 parameters)
                    bytes32 hash = keccak256(abi.encode(NESTED_TYPEHASH, user, batches, timestamp, extraParam));
                }
                
                function correctNestedOperation() external {
                    // Should NOT flag - correct usage with 3 parameters
                    bytes32 hash = keccak256(abi.encode(NESTED_TYPEHASH, user, batches, timestamp));
                }
            }
        ";

        let expected_findings = ExpectedFindings { src: 1, test: 0, ..ExpectedFindings::default() };
        expected_findings.assert_eq(content, &validate);
    }
}
