pragma solidity ^0.8.17;

import {Script} from "forge-std/Script.sol";

interface IMultiSendCallOnly {
  function multiSend(bytes memory _transactions) external payable;
}

contract CounterScript is Script {
  address public constant ZERO_ADDRESS_GOOD_NAME = address(0);

  function setUp() public {}

  function run() public {
    vm.broadcast();
  }

  function internalShouldHaveLeadingUnderscore() internal {}

  function _butInScriptsThisIsNotChecked() internal {
    uint256 _x = 1;
  }

  function thatGoesForPrivateToo() private {}
  function _soAllFourOfTheseAreAllowed() private {}
}
