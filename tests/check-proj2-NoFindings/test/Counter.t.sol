pragma solidity ^0.8.17;

import {Test} from "forge-std/Test.sol";
import "../src/Counter.sol";

contract CounterTest is Test {
  uint256 constant TEST_VAL = 123;

  Counter public counter;

  function setUp() public {
    counter = new Counter();
    counter.setNumber(0);
  }

  function test_Increment() public {
    counter.increment();
    assertEq(counter.number(), 1);
  }

  function test_SetNumber_GoodName(uint256 _x) public {
    counter.setNumber(_x);
    assertEq(counter.number(), _x);
  }

  function test_RevertIf_Overflow() public {}

  function internalShouldHaveLeadingUnderscore() internal {}

  function _butInTestsThisIsNotChecked() internal {
    uint256 _x = 1;
  }

  function thatGoesForPrivateToo() private {}
  function _soAllFourOfTheseAreAllowed() private {}
}
