// Test file showing differences between ignore directives
pragma solidity ^0.8.17;

contract CounterIgnored4 {
  // Test 1: ignore-next-item (ignores entire function declaration, even multiline)
  // scopelint: ignore-next-item
  function multiLineFunction(
    address user,
    uint256 amount
  ) internal {
    // complex function body
  }

  // Test 2: ignore-next-line (ignores only the next line)
  // scopelint: ignore-next-line
  function singleLineFunction() internal {}

  // Test 3: ignore-line (ignores only this comment line, NOT the function)
  function functionOnSameLine() internal {} // scopelint: ignore-line

  // Test 4: ignore-start/ignore-end (ignores multiple items)
  // scopelint: ignore-start
  function batchFunction1() internal {}
  function batchFunction2() private {}
  function batchFunction3() internal {}
  // scopelint: ignore-end

  // Control test: this should be flagged (no ignore directive)
  function missingLeadingUnderscoreAndNotIgnored() internal {}
} 