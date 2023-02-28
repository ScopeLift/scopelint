pragma solidity ^0.8.17;

import {Script} from "forge-std/Script.sol";

contract CounterScript is Script {
  function run() public {
    vm.broadcast();
  }

  function anotherPublic() public {
    // This method shouldn't be allowed.
  }

  function thirdPublic() external {}
}
