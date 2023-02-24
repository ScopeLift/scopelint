use crate::check::utils::{Name, VisibilitySummary};
use colored::Colorize;
use solang_parser::pt::{
    ContractDefinition, ContractPart, ContractTy, FunctionDefinition, FunctionTy, SourceUnitPart,
};
use std::{cmp::Ordering, collections::HashMap, error::Error, ffi::OsStr, fmt, fs, path::Path};
use walkdir::WalkDir;

/// Generates a specification for the current project from test names.
/// # Errors
/// Returns an error if the specification could not be generated from the Solidity code.
pub fn run() -> Result<(), Box<dyn Error>> {
    // ========================================
    // ======== Parse source contracts ========
    // ========================================
    // First, parse all source files and collect the contracts and their methods. All free functions
    // are added under a special contract called `FreeFunctions`.
    let mut src_contracts: HashMap<String, ParsedContract> = HashMap::new();
    let mut test_contracts: HashMap<String, ParsedContract> = HashMap::new();
    let mut test_contract_files: HashMap<String, Vec<ParsedContract>> = HashMap::new();

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

        let new_src_contracts = parse_contracts(&file);

        for (contract_name, contract) in new_src_contracts {
            src_contracts.insert(contract_name, contract);
        }
    }

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

        let new_test_contracts = parse_contracts(&file);
        let mut new_test_contracts_vec: Vec<ParsedContract> = Vec::new();
        for (contract_name, contract) in new_test_contracts {
            test_contracts.insert(contract_name, contract.clone());
            new_test_contracts_vec.push(contract);
        }
        test_contract_files.insert(contract_name_from_file(&file), new_test_contracts_vec);
    }

    // Debug.
    // for (contract_name, contract) in src_contracts {
    //     println!("contract {:?} has {:?} functions", contract_name, contract.functions.len());
    // }
    // for (contract_name, contract) in test_contracts {
    //     println!("contract {:?} has {:?} functions", contract_name, contract.functions.len());
    // }

    // Assumptions:
    //   - The name of a test contract file matches the name of the contract it tests.
    //   - Contracts that have names matching a function name in the source contract contain that
    //     method's tests/specification.
    let mut results: SpecResults = SpecResults::new();
    for (contract_name, test_contracts) in test_contract_files {
        // If there is no source contract with the same name, skip this test contract.
        if let Some(src_contract) = src_contracts.get(&contract_name) {
            for test_contract in test_contracts {
                // Look for the name of a function in the `src_contract` that matches the name of
                // the test contract.
                if let Some(src_contract_function) = src_contract.functions.iter().find(|f| {
                    f.name().to_ascii_lowercase() ==
                        test_contract.contract_name.to_ascii_lowercase()
                }) {
                    // Filter out internal/private functions, constructor, fallback/receive, and
                    // setUp.
                    let test_contract_spec_fns = test_contract
                        .functions
                        .iter()
                        .filter(|f| {
                            !f.is_internal_or_private() &&
                                f.name() != "setUp" &&
                                f.ty == FunctionTy::Function
                        })
                        .cloned()
                        .collect::<Vec<FunctionDefinition>>();
                    results.push_item(TestContract {
                        contract_name: test_contract.contract_name,
                        src_contract: src_contract.clone(),
                        src_contract_function: src_contract_function.clone(),
                        tests: test_contract_spec_fns,
                    });
                }
            }
        }
    }

    results.print_summary();

    Ok(())
}

fn contract_name_from_file(file: &Path) -> String {
    let file_stem = file.file_stem().unwrap().to_str().unwrap().to_string();
    file_stem[0..file_stem.len() - 2].to_string() // Slice off the ".t" at the end.
}

fn parse_contracts(file: &Path) -> HashMap<String, ParsedContract> {
    let content = fs::read_to_string(file).unwrap();
    let (pt, _comments) = solang_parser::parse(&content, 0).expect("Parsing failed");
    let mut contracts: HashMap<String, ParsedContract> = HashMap::new();

    for element in &pt.0 {
        match element {
            SourceUnitPart::FunctionDefinition(f) => {
                // These are free functions not belonging to any contract.
                contracts
                    .entry("FreeFunctions".to_string())
                    .or_insert_with(|| ParsedContract::new("FreeFunctions".to_string(), None))
                    .push_item(*f.clone());
            }
            SourceUnitPart::ContractDefinition(c) => {
                if let ContractTy::Interface(_) = c.ty {
                    continue
                }

                for el in &c.parts {
                    if let ContractPart::FunctionDefinition(f) = el {
                        contracts
                            .entry(c.name.name.clone())
                            .or_insert_with(|| {
                                ParsedContract::new(c.name.name.clone(), Some(*c.clone()))
                            })
                            .push_item(*f.clone());
                    }
                }
            }
            _ => (),
        }
    }
    contracts
}

#[derive(Clone)]
struct ParsedContract {
    // Name of the source contract.
    contract_name: String,
    // The contract item.
    contract: Option<ContractDefinition>,
    // All functions present in the source contract.
    functions: Vec<FunctionDefinition>,
}

impl ParsedContract {
    const fn new(contract_name: String, contract: Option<ContractDefinition>) -> Self {
        Self { contract_name, contract, functions: Vec::new() }
    }

    fn push_item(&mut self, function: FunctionDefinition) {
        self.functions.push(function);
    }
}

// #[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
#[derive(Clone)]
struct TestContract {
    // Name of the test contract.
    contract_name: String,
    // Associated source contract for this test contract.
    src_contract: ParsedContract,
    // The function in the source contract that is being tested.
    src_contract_function: FunctionDefinition,
    // All tests in this contract, which form the specification for the function.
    tests: Vec<FunctionDefinition>,
}

struct SpecResults {
    test_contracts: Vec<TestContract>,
}

impl SpecResults {
    fn new() -> Self {
        Self { test_contracts: Vec::new() }
    }

    fn push_item(&mut self, test_contract: TestContract) {
        self.test_contracts.push(test_contract);
    }

    fn print_summary(&self) {
        let mut sorted_test_contracts = self.test_contracts.clone();
        sorted_test_contracts
            .sort_by(|a, b| a.src_contract.contract_name.cmp(&b.src_contract.contract_name));

        println!("{}", "Protocol Specification".cyan().bold());
        for item in sorted_test_contracts {
            println!("{}.{}()", item.src_contract.contract_name, item.src_contract_function.name());

            // Remove everything before, and including, the first underscore.
            let trimmed_test_names = item.tests.iter().map(|test| {
                let fn_name = test.name();
                let trimmed_fn_name_opt = fn_name.splitn(2, '_').nth(1);
                if let Some(trimmed_fn_name) = trimmed_fn_name_opt {
                    trimmed_fn_name.to_string()
                } else {
                    // fn_name.to_string()
                    panic!("bad test name: {}", fn_name);
                }
            });

            for trimmed_fn_name in trimmed_test_names {
                // Replace underscores with colons, and camel case with spaces.
                let requirement =
                    trimmed_fn_name
                        .replace("_", ": ")
                        .chars()
                        .enumerate()
                        .map(|(i, c)| {
                            if i > 0 && c.is_uppercase() {
                                format!(" {}", c)
                            } else {
                                c.to_string()
                            }
                        })
                        .collect::<String>();
                println!("  {}", requirement);
            }
            println!("");
        }
    }
}
