// This file is identical to `Counter.sol` except it has ignore statements and adds multiline
// function declaration tests.

pragma solidity ^0.8.17;

contract CounterIgnored1 {
  uint256 public immutable _GOOD__IMMUTABLE_;
  uint256 public immutable badImmutable; // scopelint: disable-line
  // scopelint: disable-next-line
  uint256 public constant bad_constant = 1;

  uint256 public number;

  // scopelint: disable-next-item
  constructor() {
    _GOOD__IMMUTABLE_ = 2000;
    badImmutable = 5;
  }

  function setNumber(uint256 newNumber) public {
    number = newNumber;
  }

  function increment() public {
    number++;
  }

  // scopelint: disable-next-line
  function internalShouldHaveLeadingUnderscore() internal {}
  function _internalHasLeadingUnderscore() internal {}
  function privateShouldHaveLeadingUnderscore() private {} // scopelint: disable-line

  function _privateHasLeadingUnderscore() private {
    number += 1000;
  }

  // -------- Multiline function declaration tests --------
  function prettyLongMethodName( // scopelint: disable-line
    address someImportantUser,
    uint256 someImportantNumber,
    bytes32 someImportantData
  ) internal {
    // do something
  }

  // scopelint: disable-next-line
  function prettyLongMethodName2(
    address someImportantUser,
    uint256 someImportantNumber,
    bytes32 someImportantData
  ) internal {
    // do something
  }
}
