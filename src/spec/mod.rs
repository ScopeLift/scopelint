// The `.extension()` method only looks after the last dot in the file name, so it will return
// Some("sol") for both "Foo.sol" and "Foo.t.sol". This is not what we want here, so we just check
// extensions manually with `ends_with`.
#![allow(clippy::case_sensitive_file_extension_comparisons)]

use crate::check::utils::{Name, VisibilitySummary};
use colored::Colorize;
use solang_parser::pt::{
    ContractDefinition, ContractPart, ContractTy, FunctionDefinition, SourceUnitPart,
};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

/// Generates a specification for the current project from test names.
/// # Errors
/// Returns an error if the specification could not be generated from the Solidity code.
/// # Panics
/// Panics when a file path could not be unwrapped.
pub fn run(show_internal: bool) -> Result<(), Box<dyn Error>> {
    // =================================
    // ======== Parse contracts ========
    // =================================

    // First, parse all source and test files to collect the contracts and their methods. All free
    // functions are added under a special contract called `FreeFunctions`.
    let src_contracts = get_contracts_for_dir("./src", ".sol", show_internal);
    let test_contracts = get_contracts_for_dir("./test", ".t.sol", show_internal);

    // ========================================
    // ======== Generate Specification ========
    // ========================================

    // Now we generate contract specifications from the test contracts.
    // Assumptions:
    //   - The name of a test contract file matches the name of the contract it tests.
    //   - If the name of a test contract matches a function name in the source contract, that test
    //     contract contains that source method's tests/specification.
    let mut protocol_spec = ProtocolSpecification::new();
    for src_contract in src_contracts {
        // Skip contracts with no functions - they have nothing to specify
        if src_contract.functions.is_empty() {
            continue;
        }

        let mut contract_specification = ContractSpecification::new(src_contract.clone());
        let src_contract_name = src_contract.contract.unwrap().name.unwrap().name;

        for test_contract in &test_contracts {
            if src_contract_name == test_contract.contract_name_from_file() {
                contract_specification.push_test_contract(test_contract.clone());
            }
        }
        protocol_spec.push_contract_specification(contract_specification);
    }
    protocol_spec.print_summary();

    Ok(())
}

#[derive(Clone)]
struct ParsedContract {
    // Path to the contract file.
    path: PathBuf,
    // The contract item, or `None` for free functions.
    contract: Option<ContractDefinition>,
    // All functions present in the contract.
    functions: Vec<FunctionDefinition>,
}

impl ParsedContract {
    fn new(path: PathBuf, contract: Option<ContractDefinition>, show_internal: bool) -> Self {
        let functions =
            contract.as_ref().map_or(Vec::new(), |c| get_functions_from_contract(c, show_internal));
        Self { path, contract, functions }
    }

    fn contract_name(&self) -> String {
        self.contract
            .as_ref()
            .map_or_else(|| "FreeFunctions".to_string(), |c| c.name.as_ref().unwrap().name.clone())
    }

    fn contract_name_from_file(&self) -> String {
        let file_stem = self.path.file_stem().unwrap().to_str().unwrap().to_string();
        if file_stem.ends_with(".t") {
            // Get everything before the first dot, slicing off `.t`. This enables support for both
            // (1) putting all tests in MyContract.t.sol, and (2) splitting up tests across multiple
            // files such as `MyContract.SomeFunction.t.sol`.
            file_stem.split('.').next().unwrap().to_string()
        } else {
            file_stem
        }
    }
}

struct ContractSpecification {
    src_contract: ParsedContract,
    test_contracts: Vec<ParsedContract>,
}

impl ContractSpecification {
    const fn new(src_contract: ParsedContract) -> Self {
        Self { src_contract, test_contracts: Vec::new() }
    }

    fn push_test_contract(&mut self, test_contract: ParsedContract) {
        self.test_contracts.push(test_contract);
    }

    fn print_specification(&self) {
        let prefix = format!("\n{}", "Contract Specification:".bold());
        let contract_name = format!("{}", self.src_contract.contract_name().bold());
        println!("{prefix} {contract_name}");

        // Vectors of functions are already sorted by their order of appearance in the source code,
        // which is the order we want to print in.
        let src_fns = &self.src_contract.functions;
        let num_src_fns = src_fns.len();

        for (i, src_fn) in src_fns.iter().enumerate() {
            let src_fn_name_prefix = if i == num_src_fns - 1 { "└── " } else { "├── " };

            self.test_contracts
                .iter()
                .find(|tc| {
                    // Find the test contract with the same name
                    tc.contract_name().eq_ignore_ascii_case(&src_fn.name())
                })
                .map_or_else(
                    // If there's no matching test contract, print the name of the source function
                    // in red to indicate to the user that it is missing tests
                    // to define it's requirements. Otherwise, parse the test
                    // names into a specification and print it.
                    || println!("{src_fn_name_prefix}{}", src_fn.name().red()),
                    |test_contract| {
                        println!("{src_fn_name_prefix}{}", src_fn.name());

                        let test_fns = &test_contract.functions;
                        let num_test_fns = test_fns.len();
                        for (j, f) in test_fns.iter().enumerate() {
                            let is_test_fn =
                                f.is_public_or_external() && f.name().starts_with("test");
                            if !is_test_fn {
                                continue;
                            }

                            let test_fn_name_prefix =
                                if i < num_src_fns - 1 && j == num_test_fns - 1 {
                                    "│   └── "
                                } else if i < num_src_fns - 1 {
                                    "│   ├── "
                                } else if j == num_test_fns - 1 {
                                    "    └── "
                                } else {
                                    "    ├── "
                                };

                            // Remove everything before, and including, the first underscore.
                            let fn_name = f.name();
                            let trimmed_fn_name_opt = fn_name.split_once('_').map(|x| x.1);

                            // If there were no underscores present this is an invalid test name, so
                            // we print nothing. The user should use `scopelint check` to make sure
                            // all test names are valid. Otherwise, parse and print the
                            // requirement.
                            if let Some(trimmed_fn_name) = trimmed_fn_name_opt {
                                let requirement = trimmed_fn_name_to_requirement(trimmed_fn_name);
                                println!("{test_fn_name_prefix}{requirement}");
                            }
                        }
                    },
                );
        }
    }
}

struct ProtocolSpecification {
    contract_specifications: Vec<ContractSpecification>,
}

impl ProtocolSpecification {
    const fn new() -> Self {
        Self { contract_specifications: Vec::new() }
    }

    fn push_contract_specification(&mut self, contract_specification: ContractSpecification) {
        self.contract_specifications.push(contract_specification);
    }

    fn print_summary(&self) {
        for contract_specification in &self.contract_specifications {
            contract_specification.print_specification();
        }
    }
}

// ==================================
// ======== Helper functions ========
// ==================================

fn get_contracts_for_dir<P: AsRef<Path>>(
    dir: P,
    extension: &str,
    show_internal: bool,
) -> Vec<ParsedContract> {
    let mut contracts: Vec<ParsedContract> = Vec::new();
    for result in WalkDir::new(dir) {
        let dent = match result {
            Ok(dent) => dent,
            Err(err) => {
                eprintln!("{err}");
                continue;
            }
        };

        let file = dent.path();
        if !dent.file_type().is_file() || !dent.path().to_str().unwrap().ends_with(extension) {
            continue;
        }

        let new_contracts = parse_contracts(file, show_internal);
        contracts.extend(new_contracts);
    }
    contracts
}

fn parse_contracts(file: &Path, show_internal: bool) -> Vec<ParsedContract> {
    let content = fs::read_to_string(file).unwrap();
    let (pt, _comments) = crate::parser::parse_solidity(&content, 0).expect("Parsing failed");
    let mut contracts: Vec<ParsedContract> = Vec::new();

    for element in &pt.0 {
        match element {
            SourceUnitPart::FunctionDefinition(_f) => {
                // These are free functions not belonging to any contract.
                todo!("Free functions not yet supported.");
            }
            SourceUnitPart::ContractDefinition(c) => {
                if let ContractTy::Interface(_) = c.ty {
                    continue;
                }

                contracts.push(ParsedContract::new(
                    file.to_path_buf(),
                    Some(*c.clone()),
                    show_internal,
                ));
            }
            _ => (),
        }
    }
    contracts
}

fn get_functions_from_contract(
    contract: &ContractDefinition,
    show_internal: bool,
) -> Vec<FunctionDefinition> {
    let mut functions = Vec::new();
    for element in &contract.parts {
        if let ContractPart::FunctionDefinition(f) = element {
            if show_internal || f.is_public_or_external() {
                functions.push(*f.clone());
            }
        }
    }
    functions
}

fn trimmed_fn_name_to_requirement(trimmed_fn_name: &str) -> String {
    // Replace underscores with colons, and camel case with spaces.
    trimmed_fn_name
        .replace('_', ":")
        .chars()
        .map(|c| if c.is_uppercase() { format!(" {c}") } else { c.to_string() })
        .collect::<String>()
}
