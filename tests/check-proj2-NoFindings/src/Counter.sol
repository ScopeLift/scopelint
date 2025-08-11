// SPDX-License-Identifier: MIT
pragma solidity ^0.8.17;

contract Counter {
  uint256 public immutable GOOD_IMMUTABLE;
  uint256 public constant GOOD_CONSTANT__ = 1;

  uint256 public number;

  constructor() {
    GOOD_IMMUTABLE = 2000;
  }

  function setNumber(uint256 _newNumber) public {
    number = _newNumber;
  }

  function increment() public {
    number++;
  }

  function _internalHasLeadingUnderscore() internal {
    number += 1000;
  }

  function _privateHasLeadingUnderscore() private {}
}
