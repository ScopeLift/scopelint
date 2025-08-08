/// Ultimately what we care about is that the user sees the correct spec in their terminal.
/// Therefore, most testing is done by running the binary against a sample forge project and
/// checking the output.
use std::{
    env,
    process::{Command, Output},
};

fn run_scopelint(test_folder: &str) -> Output {
    let cwd = env::current_dir().unwrap();
    let project_path = cwd.join("tests").join(test_folder);
    let binary_path = cwd.join("target/debug/scopelint");

    Command::new(binary_path)
        .current_dir(project_path)
        .arg("spec")
        .output()
        .expect("Failed to execute command")
}

#[test]
fn test_spec_proj1() {
    let output = run_scopelint("spec-proj1");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected_spec = r#"
Contract Specification: ERC20
├── constructor
│   ├──  Stored Name Matches Constructor Input
│   ├──  Stored Symbol Matches Constructor Input
│   ├──  Stored Decimals Matches Constructor Input
│   ├──  Sets Initial Chain Id
│   └──  Sets Initial Domain Separator
├── approve
│   ├──  Sets Allowance Mapping To Approved Amount
│   ├──  Returns True For Successful Approval
│   └──  Emits Approval Event
├── transfer
│   ├──  Revert If: Spender Has Insufficient Balance
│   ├──  Does Not Change Total Supply
│   ├──  Increases Recipient Balance By Sent Amount
│   ├──  Decreases Sender Balance By Sent Amount
│   ├──  Returns True
│   └──  Emits Transfer Event
├── transferFrom
├── permit
├── DOMAIN_SEPARATOR
├── computeDomainSeparator
├── _mint
└── _burn
"#;
    assert_eq!(stdout, expected_spec);
}

#[test]
fn test_spec_proj2_empty_contract() {
    let output = run_scopelint("spec-proj2-EmptyContract");
    let stdout = String::from_utf8(output.stdout).unwrap();
    // Empty contracts should be ignored and produce no output
    let expected_spec = "";
    assert_eq!(stdout, expected_spec);
}
