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
fn test_check_proj1_all_findings() {
    let output = run_scopelint("proj1-AllFindings");
    let stderr = String::from_utf8(output.stderr).unwrap();
    let findings: Vec<&str> = stderr.split("\n").collect();

    let expected_findings = [
        "Invalid constant or immutable name in ./src/Counter.sol on line 5: badImmutable",
        "Invalid constant or immutable name in ./src/Counter.sol on line 6: bad_constant",
        "Invalid src method name in ./src/Counter.sol on line 23: internalShouldHaveLeadingUnderscore",
        "Invalid src method name in ./src/Counter.sol on line 25: privateShouldHaveLeadingUnderscore",
        "Invalid constant or immutable name in ./script/ScriptHelpers.sol on line 4: stillNeedGoodNames",
        r#"Invalid script interface in ./script/Counter2.s.sol: Scripts must have a single public method named `run` (excluding `setUp`), but the following methods were found: ["run", "anotherPublic", "thirdPublic"]"#,
        "Invalid constant or immutable name in ./script/Counter.s.sol on line 6: bad_constant",
        "Invalid constant or immutable name in ./script/Counter.s.sol on line 7: VERY_bad_constant",
        "Invalid constant or immutable name in ./script/Counter.s.sol on line 8: sorryBadName",
        r#"Invalid script interface in ./script/Counter.s.sol: Scripts must have a single public method named `run` (excluding `setUp`), but the following methods were found: ["run", "runExternal"]"#,
        "Invalid constant or immutable name in ./test/Counter.t.sol on line 7: testVal",
        "Invalid test name in ./test/Counter.t.sol on line 16: testIncrementBadName",
        "error: Convention checks failed, see details above",
        ""
    ];

    for (i, expected) in expected_findings.iter().enumerate() {
        assert_eq!(findings[i], *expected);
    }
}
#[test]
fn test_check_proj2_no_findings() {
    let output = run_scopelint("proj2-NoFindings");
    let stderr = String::from_utf8(output.stderr).unwrap();
    let findings: Vec<&str> = stderr.split("\n").collect();

    let expected_findings = [""];

    for (i, expected) in expected_findings.iter().enumerate() {
        assert_eq!(findings[i], *expected);
    }
}
