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
