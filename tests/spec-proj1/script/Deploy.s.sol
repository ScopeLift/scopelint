// SPDX-License-Identifier: UNLICENSED

pragma solidity ^0.8.17;

import "forge-std/Script.sol";
import {ERC20} from "src/ERC20.sol";

contract Deploy is Script {
  ERC20 token;

  function run() public {
    vm.broadcast();
    token = new ERC20("Token", "TKN", 18);
  }
}
