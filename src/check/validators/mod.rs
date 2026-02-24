/// Validates that Solidity and TOML files are formatted correctly.
pub mod formatting;

/// Validates that constant and immutable variable names are in `ALL_CAPS`.
pub mod constant_names;

/// Validates that a script has a single public method named `run`.
pub mod script_has_public_run_method;

/// Validates that internal and private function names are prefixed with an underscore.
pub mod src_names_internal;

/// Validates that test names are in the correct format.
pub mod test_names;

/// Validates that source files have SPDX license headers.
pub mod src_spdx_header;

/// Validates that variable names follow the correct naming conventions.
pub mod variable_names;

/// Validates that error names are prefixed with `ContractName_`
pub mod error_prefix;

/// Validates that EIP712 typehashes match their corresponding struct definitions.
pub mod eip712_typehash;

/// Validates that all imported symbols are actually used in the file.
pub mod unused_imports;
