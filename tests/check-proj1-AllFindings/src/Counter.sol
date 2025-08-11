pragma solidity ^0.8.17;

contract Counter {
  uint256 public immutable _GOOD__IMMUTABLE_;
  uint256 public immutable badImmutable;
  uint256 public constant bad_constant = 1;

  bytes32 public constant PERMIT_TYPEHASH = keccak256("Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)");

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
  function permit(address owner, address spender, uint256 value) external pure {
    keccak256(abi.encode(PERMIT_TYPEHASH, owner, spender, value));
  }

  // Invalid event - should be prefixed with "Counter_"
  event InvalidEvent(uint256 value);
  event AnotherInvalidEvent();
}

// scopelint: this directive is invalid
// Extra line break at the bottom is intention to ensure formatting fails.

