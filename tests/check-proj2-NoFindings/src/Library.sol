// SPDX-License-Identifier: MIT
pragma solidity ^0.8.17;

library MyLibrary {
  function internalNeedsNoUnderscores() internal pure returns (uint256) {
    return 1;
  }

  function privateNeedsNoUnderscores() external pure returns (uint256) {
    return 1;
  }
}
