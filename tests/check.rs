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
    let binary_path = cwd.join("target/debug/scopelint");

    Command::new(binary_path)
        .current_dir(project_path)
        .arg("check")
        .output()
        .expect("Failed to execute command")
}

#[test]
fn test_check_proj1_all_findings() {
    let output = run_scopelint("check-proj1-AllFindings");
    let stderr = String::from_utf8(output.stderr).unwrap();
    let findings: Vec<&str> = stderr.split("\n").collect();

    let expected_findings = [
        "Invalid constant or immutable name in ./script/Counter.s.sol on line 7: VERY_bad_constant",
        "Invalid constant or immutable name in ./script/Counter.s.sol on line 6: bad_constant",
        "Invalid constant or immutable name in ./script/Counter.s.sol on line 8: sorryBadName",
        "Invalid constant or immutable name in ./script/ScriptHelpers.sol on line 4: stillNeedGoodNames",
        "Invalid constant or immutable name in ./src/Counter.sol on line 5: badImmutable",
        "Invalid constant or immutable name in ./src/Counter.sol on line 6: bad_constant",
        "Invalid constant or immutable name in ./test/Counter.t.sol on line 7: testVal",
        "Invalid src method name in ./src/Counter.sol on line 1: Missing SPDX-License-Identifier header",
        "Invalid src method name in ./src/Counter.sol on line 23: internalShouldHaveLeadingUnderscore",
        "Invalid src method name in ./src/Counter.sol on line 25: privateShouldHaveLeadingUnderscore",
        "Invalid src method name in ./src/CounterIgnored1.sol on line 1: Missing SPDX-License-Identifier header",
        "Invalid src method name in ./src/CounterIgnored2.sol on line 1: Missing SPDX-License-Identifier header",
        "Invalid src method name in ./src/CounterIgnored3.sol on line 1: Missing SPDX-License-Identifier header",
        "Invalid src method name in ./src/CounterIgnored4.sol on line 1: Missing SPDX-License-Identifier header",
        "Invalid src method name in ./src/CounterIgnored4.sol on line 29: missingLeadingUnderscoreAndNotIgnored",
        "Invalid test name in ./test/Counter.t.sol on line 16: testIncrementBadName",
        "Invalid directive in ./src/Counter.sol: Invalid inline config item: this directive is invalid",
        "Invalid variable name in ./script/Counter.s.sol on line 25: Local variable 'x' should have underscore prefix",
        "Invalid variable name in ./src/Counter.sol on line 15: Parameter 'newNumber' should have underscore prefix",
        "Invalid variable name in ./src/Counter.sol on line 4: State variable '_GOOD__IMMUTABLE_' should NOT have underscore prefix",
        "Invalid variable name in ./src/CounterIgnored3.sol on line 20: Parameter 'newNumber' should have underscore prefix",
        "Invalid variable name in ./src/CounterIgnored3.sol on line 41: Parameter 'someImportantData' should have underscore prefix",
        "Invalid variable name in ./src/CounterIgnored3.sol on line 50: Parameter 'someImportantData' should have underscore prefix",
        "Invalid variable name in ./src/CounterIgnored3.sol on line 40: Parameter 'someImportantNumber' should have underscore prefix",
        "Invalid variable name in ./src/CounterIgnored3.sol on line 49: Parameter 'someImportantNumber' should have underscore prefix",
        "Invalid variable name in ./src/CounterIgnored3.sol on line 39: Parameter 'someImportantUser' should have underscore prefix",
        "Invalid variable name in ./src/CounterIgnored3.sol on line 48: Parameter 'someImportantUser' should have underscore prefix",
        "Invalid variable name in ./src/CounterIgnored3.sol on line 7: State variable '_GOOD__IMMUTABLE_' should NOT have underscore prefix",
        "Invalid variable name in ./test/Counter.t.sol on line 31: Local variable 'x' should have underscore prefix",
        "Invalid variable name in ./test/Counter.t.sol on line 21: Parameter 'x' should have underscore prefix",
        "Invalid event name in ./src/Counter.sol on line 33: Event 'AnotherInvalidEvent' should be prefixed with 'Counter_'",
        "Invalid event name in ./src/Counter.sol on line 32: Event 'InvalidEvent' should be prefixed with 'Counter_'",
        "error: Convention checks failed, see details above",
        "error: Formatting validation failed, run `scopelint fmt` to fix",
        "",
    ];

    for (i, expected) in expected_findings.iter().enumerate() {
        assert_eq!(findings[i], *expected);
    }
    assert_eq!(findings.len(), expected_findings.len());
}

#[test]
fn test_check_proj2_no_findings() {
    let output = run_scopelint("check-proj2-NoFindings");
    let stderr = String::from_utf8(output.stderr).unwrap();
    let findings: Vec<&str> = stderr.split("\n").collect();

    let expected_findings = [""];

    for (i, expected) in expected_findings.iter().enumerate() {
        assert_eq!(findings[i], *expected);
    }
    assert_eq!(findings.len(), expected_findings.len());
}
