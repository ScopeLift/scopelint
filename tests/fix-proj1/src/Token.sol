// SPDX-License-Identifier: MIT
pragma solidity ^0.8.17;

import { ERC20 } from "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract Token {
    ERC20 public token;

    function setToken(ERC20 _token) public {
        token = _token;
    }
}
