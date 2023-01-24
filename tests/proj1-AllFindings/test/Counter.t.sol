pragma solidity ^0.8.17;

import {Test} from "forge-std/Test.sol";
import "../src/Counter.sol";

contract CounterTest is Test {
  uint256 constant testVal = 123;

  Counter public counter;

  function setUp() public {
    counter = new Counter();
    counter.setNumber(0);
  }

  function testIncrementBadName() public {
    counter.increment();
    assertEq(counter.number(), 1);
  }

  function test_SetNumber_GoodName(uint256 x) public {
    counter.setNumber(x);
    assertEq(counter.number(), x);
  }

  function test_RevertIf_Overflow() public {}

  function internalShouldHaveLeadingUnderscore() internal {}

  function _butInTestsThisIsNotChecked() internal {
    uint256 x = 1;
  }

  function thatGoesForPrivateToo() private {}
  function _soAllFourOfTheseAreAllowed() private {}
}
