pragma solidity ^0.8.17;

contract Counter {
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

  function badLocalVars(uint256 param1, address param2) public returns (uint256 result) {
    uint256 localVar = 42;
    address localAddr = address(0);
    
    for (uint256 i = 0; i < 10; i++) {
      uint256 temp = i * 2;
    }
    
    if (localVar > 0) {
      uint256 anotherVar = localVar * 2;
    }
    
    return localVar;
  }

  function goodLocalVars(uint256 _param1, address _param2) public returns (uint256 _result) {
    uint256 _localVar = 42;
    address _localAddr = address(0);
    
    for (uint256 _i = 0; _i < 10; _i++) {
      uint256 _temp = _i * 2;
    }
    
    if (_localVar > 0) {
      uint256 _anotherVar = _localVar * 2;
    }
    
    return _localVar;
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

// scopelint: this directive is invalid
// Extra line break at the bottom is intention to ensure formatting fails.

