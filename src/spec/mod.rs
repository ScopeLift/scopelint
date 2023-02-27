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
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

/// Generates a specification for the current project from test names.
/// # Errors
/// Returns an error if the specification could not be generated from the Solidity code.
/// # Panics
/// Panics when a file path could not be unwrapped.
pub fn run() -> Result<(), Box<dyn Error>> {
    // ========================================
    // ======== Parse source contracts ========
    // ========================================

    // First, parse all source files and collect the contracts and their methods. All free functions
    // are added under a special contract called `FreeFunctions`.
    let mut src_contracts: Vec<ParsedContract> = Vec::new();
    let mut test_contracts: Vec<ParsedContract> = Vec::new();

    for result in WalkDir::new("./src") {
        let dent = match result {
            Ok(dent) => dent,
            Err(err) => {
                eprintln!("{err}");
                continue
            }
        };

        let file = dent.path();
        if !dent.file_type().is_file() || file.extension() != Some(OsStr::new("sol")) {
            continue
        }

        let new_src_contracts = parse_contracts(file);
        src_contracts.extend(new_src_contracts);
    }

    // ======================================
    // ======== Parse Test contracts ========
    // ======================================

    // Next we do the same thing for all test contracts.
    for result in WalkDir::new("./tests") {
        let dent = match result {
            Ok(dent) => dent,
            Err(err) => {
                eprintln!("{err}");
                continue
            }
        };

        let file = dent.path();
        if !dent.file_type().is_file() || !dent.path().to_str().unwrap().ends_with(".t.sol") {
            continue
        }

        let new_test_contracts = parse_contracts(file);
        test_contracts.extend(new_test_contracts);
    }

    // ========================================
    // ======== Generate Specification ========
    // ========================================

    // Now we generate contract specifications from the test contracts.
    // Assumptions:
    //   - The name of a test contract file matches the name of the contract it tests.
    //   - Contracts that have names matching a function name in the source contract contain that
    //     method's tests/specification.
    let mut protocol_spec = ProtocolSpecification::new();

    for src_contract in src_contracts {
        let mut contract_specification = ContractSpecification::new(src_contract.clone());
        let src_contract_name = src_contract.contract.unwrap().name.name;

        for test_contract in &test_contracts {
            // If the name of the source contract matches the file name of the test contract, add
            // the test contract.
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
    // Path to the source file.
    path: PathBuf,
    // The contract item.
    contract: Option<ContractDefinition>,
    // All functions present in the source contract.
    functions: Vec<FunctionDefinition>,
}

impl ParsedContract {
    fn new(path: PathBuf, contract: Option<ContractDefinition>) -> Self {
        // TODO Clippy bug giving false redundant_closure warning.
        #[allow(clippy::redundant_closure)]
        let functions = contract.as_ref().map_or_else(|| Vec::new(), get_functions_from_contract);
        Self { path, contract, functions }
    }

    fn contract_name(&self) -> String {
        self.contract.as_ref().map_or_else(|| "FreeFunctions".to_string(), |c| c.name.name.clone())
    }

    fn contract_name_from_file(&self) -> String {
        let file_stem = self.path.file_stem().unwrap().to_str().unwrap().to_string();
        if file_stem.ends_with(".t") {
            file_stem[0..file_stem.len() - 2].to_string() // Slice off the ".t" at the end.
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
            // Find the test contract with the same name
            let test_contract = self.test_contracts.iter().find(|tc| {
                tc.contract_name().to_ascii_lowercase() == src_fn.name().to_ascii_lowercase()
            });

            let src_fn_name_prefix = if i == num_src_fns - 1 { "└── " } else { "├── " };

            test_contract.map_or_else(
                || println!("{src_fn_name_prefix}{}", src_fn.name().red()),
                |test_contract| {
                    println!("{src_fn_name_prefix}{}", src_fn.name());

                    let test_fns = &test_contract.functions;
                    let num_test_fns = test_fns.len();
                    for (j, f) in test_fns.iter().enumerate() {
                        let is_test_fn = f.is_public_or_external() && f.name().starts_with("test");
                        if !is_test_fn {
                            continue
                        }

                        let test_fn_name_prefix = if i < num_src_fns - 1 && j == num_test_fns - 1 {
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
                        if let Some(trimmed_fn_name) = trimmed_fn_name_opt {
                            // Replace underscores with colons, and camel case with spaces.
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

fn parse_contracts(file: &Path) -> Vec<ParsedContract> {
    let content = fs::read_to_string(file).unwrap();
    let (pt, _comments) = solang_parser::parse(&content, 0).expect("Parsing failed");
    let mut contracts: Vec<ParsedContract> = Vec::new();

    for element in &pt.0 {
        match element {
            SourceUnitPart::FunctionDefinition(_f) => {
                // These are free functions not belonging to any contract.
                todo!("Free functions not yet supported.");
            }
            SourceUnitPart::ContractDefinition(c) => {
                if let ContractTy::Interface(_) = c.ty {
                    continue
                }

                contracts.push(ParsedContract::new(file.to_path_buf(), Some(*c.clone())));
            }
            _ => (),
        }
    }
    contracts
}

fn get_functions_from_contract(contract: &ContractDefinition) -> Vec<FunctionDefinition> {
    let mut functions = Vec::new();
    for element in &contract.parts {
        if let ContractPart::FunctionDefinition(f) = element {
            functions.push(*f.clone());
        }
    }
    functions
}

fn trimmed_fn_name_to_requirement(trimmed_fn_name: &str) -> String {
    // Replace underscores with colons, and camel case with spaces.
    trimmed_fn_name
        .replace('_', ":")
        .chars()
        .enumerate()
        .map(|(_i, c)| if c.is_uppercase() { format!(" {c}") } else { c.to_string() })
        .collect::<String>()
}
