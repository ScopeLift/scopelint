/// Ultimately what we care about is that the user sees details on all failed checks in their
/// terminal. Therefore, most testing is done by running the binary against a sample forge
/// project and checking the output.
use std::{
    env,
    process::{Command, Output},
};

fn run_scopelint(test_folder: &str) -> Output {
    let cwd = env::current_dir().unwrap();
    let project_path = cwd.join("tests").join(test_folder);

    Command::new("../../target/debug/scopelint")
        .current_dir(project_path)
        .arg("check")
        .output()
        .expect("Failed to execute command")
}

#[test]
fn test_check_proj1() {
    let output = run_scopelint("proj1-AllFindings");
    let stderr = String::from_utf8(output.stderr).unwrap();
    let findings: Vec<&str> = stderr.split("\n").collect();

    let expected_findings = [
        "Invalid constant or immutable name in ./src/Counter.sol on line 6: bad_constant",
        "Invalid test name in ./test/Counter.t.sol on line 15: testIncrement",
        "Invalid test name in ./test/Counter.t.sol on line 20: testSetNumber",
        "error: Convention checks failed, see details above",
        "",
    ];

    for (i, expected) in expected_findings.iter().enumerate() {
        assert_eq!(findings[i], *expected);
    }
}
