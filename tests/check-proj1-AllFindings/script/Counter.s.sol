pragma solidity ^0.8.17;

import {Script} from "forge-std/Script.sol";

contract CounterScript is Script {
  uint256 public constant bad_constant = 1;
  uint256 internal constant VERY_bad_constant = 2;
  uint256 public constant sorryBadName = 3;
  address public constant ZERO_ADDRESS_GOOD_NAME = address(0);

  function setUp() public {}

  function run() public {
    vm.broadcast();
  }

  function runExternal() external {
    // This method shouldn't be allowed.
    // More comments.
  }

  function internalShouldHaveLeadingUnderscore() internal {}

  function _butInScriptsThisIsNotChecked() internal {
    uint256 x = 1;
  }

  function thatGoesForPrivateToo() private {}
  function _soAllFourOfTheseAreAllowed() private {}
}
