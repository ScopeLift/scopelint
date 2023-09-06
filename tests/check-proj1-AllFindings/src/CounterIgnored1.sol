// This file is identical to `Counter.sol`, except it has an ignore statement below to disable
// scopelint checks on the whole file instead of only on specific lines.

// scopelint: disable-start
pragma solidity ^0.8.17;

contract CounterIgnored1 {
  uint256 public immutable _GOOD__IMMUTABLE_;
  uint256 public immutable badImmutable;
  uint256 public constant bad_constant = 1;

  uint256 public number;

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

  function internalShouldHaveLeadingUnderscore() internal {}
  function _internalHasLeadingUnderscore() internal {}
  function privateShouldHaveLeadingUnderscore() private {}

  function _privateHasLeadingUnderscore() private {
    number += 1000;
  }
}
