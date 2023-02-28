// SPDX-License-Identifier: AGPL-3.0-only
pragma solidity ^0.8.17;

// Modified from Solmate at the below commit, with `test_RevertIf_` usage replaced with
// `vm.expectRevert`
// https://github.com/transmissions11/solmate/blob/1b3adf677e7e383cc684b5d5bd441da86bf4bf1c/src/test/ERC20.t.sol

import {stdError, Test} from "forge-std/Test.sol";
import {MockERC20} from "test/mocks/MockERC20.sol";

contract ERC20Test is Test {
  MockERC20 token;

  bytes32 constant PERMIT_TYPEHASH =
    keccak256("Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)");

  event Transfer(address indexed from, address indexed to, uint256 amount);
  event Approval(address indexed owner, address indexed spender, uint256 amount);

  function setUp() public virtual {
    token = new MockERC20("Token", "TKN", 18);
  }
}

contract Constructor is ERC20Test {
  function test_StoredNameMatchesConstructorInput() public {
    assertEq(token.name(), "Token");
  }

  function test_StoredSymbolMatchesConstructorInput() public {
    assertEq(token.symbol(), "TKN");
  }

  function test_StoredDecimalsMatchesConstructorInput() public {
    assertEq(token.decimals(), 18);
  }

  function test_SetsInitialChainId() public {
    assertEq(token.exposed_INITIAL_CHAIN_ID(), block.chainid);
  }

  function test_SetsInitialDomainSeparator() public {
    assertEq(
      token.exposed_INITIAL_DOMAIN_SEPARATOR(),
      hex"aa9e832a9ef68cda525e9c935a73b5f1f4a30877cfb71ae89cf97bb686658b2c"
    );
  }
}

contract Approve is ERC20Test {
  function test_SetsAllowanceMappingToApprovedAmount() public {
    token.approve(address(0xBEEF), 1e18);
    assertEq(token.allowance(address(this), address(0xBEEF)), 1e18);
  }

  function test_ReturnsTrueForSuccessfulApproval() public {
    assertTrue(token.approve(address(0xBEEF), 1e18));
  }

  function test_EmitsApprovalEvent() public {
    vm.expectEmit(true, true, true, true);
    emit Approval(address(this), address(0xBEEF), 1e18);
    token.approve(address(0xBEEF), 1e18);
  }
}

contract Transfer is ERC20Test {
  function setUp() public override {
    ERC20Test.setUp();
    token.mint(address(this), 1e18);
  }

  function test_RevertIf_SpenderHasInsufficientBalance() public {
    vm.expectRevert(stdError.arithmeticError);
    token.transfer(address(0xBEEF), 2e18);
  }

  function test_DoesNotChangeTotalSupply() public {
    uint256 initTotalSupply = token.totalSupply();
    token.transfer(address(0xBEEF), 1e18);
    assertEq(token.totalSupply(), initTotalSupply);
  }

  function test_IncreasesRecipientBalanceBySentAmount() public {
    uint256 initRecipientBalance = token.balanceOf(address(0xBEEF));
    token.transfer(address(0xBEEF), 1e18);
    assertEq(token.balanceOf(address(0xBEEF)), initRecipientBalance + 1e18);
  }

  function test_DecreasesSenderBalanceBySentAmount() public {
    uint256 initSenderBalance = token.balanceOf(address(this));
    token.transfer(address(0xBEEF), 1e18);
    assertEq(token.balanceOf(address(this)), initSenderBalance - 1e18);
  }

  function test_ReturnsTrue() public {
    assertTrue(token.transfer(address(0xBEEF), 1e18));
  }

  function test_EmitsTransferEvent() public {
    vm.expectEmit(true, true, true, true);
    emit Transfer(address(this), address(0xBEEF), 1e18);
    token.transfer(address(0xBEEF), 1e18);
  }
}

// contract TransferFrom is ERC20Test {
//   function test_RevertIf_TransferFromInsufficientAllowance() public {
//     address from = address(0xABCD);

//     token.mint(from, 1e18);

//     vm.prank(from);
//     token.approve(address(this), 0.9e18);
//     vm.expectRevert(stdError.arithmeticError);
//     token.transferFrom(from, address(0xBEEF), 1e18);
//   }

//   function test_TransferFrom() public {
//     address from = address(0xABCD);

//     token.mint(from, 1e18);

//     vm.prank(from);
//     token.approve(address(this), 1e18);

//     assertTrue(token.transferFrom(from, address(0xBEEF), 1e18));
//     assertEq(token.totalSupply(), 1e18);

//     assertEq(token.allowance(from, address(this)), 0);

//     assertEq(token.balanceOf(from), 0);
//     assertEq(token.balanceOf(address(0xBEEF)), 1e18);
//   }

//   function test_InfiniteApproveTransferFrom() public {
//     address from = address(0xABCD);

//     token.mint(from, 1e18);

//     vm.prank(from);
//     token.approve(address(this), type(uint256).max);

//     assertTrue(token.transferFrom(from, address(0xBEEF), 1e18));
//     assertEq(token.totalSupply(), 1e18);

//     assertEq(token.allowance(from, address(this)), type(uint256).max);

//     assertEq(token.balanceOf(from), 0);
//     assertEq(token.balanceOf(address(0xBEEF)), 1e18);
//   }
// }

// contract Permit is ERC20Test {}

// contract DOMAIN_SEPARATOR is ERC20Test {}

// contract ComputeDomainSeparator is ERC20Test {}

// contract _mint is ERC20Test {
//   function test_IncreasesTotalSupplyByMintAmount() public {
//     assertEq(token.totalSupply(), 0);
//     token.mint(address(0xBEEF), 1e18);
//     assertEq(token.totalSupply(), 1e18);
//     assertEq(token.balanceOf(address(0xBEEF)), 1e18);
//   }

//   function test_IncreasesRecipientBalanceByMintAmount() public {
//     assertEq(token.balanceOf(address(0xBEEF)), 0);
//     token.mint(address(0xBEEF), 1e18);
//     assertEq(token.balanceOf(address(0xBEEF)), 1e18);
//   }
// }

// contract _burn is ERC20Test {
//   function test_Burn() public {
//     token.mint(address(0xBEEF), 1e18);
//     token.burn(address(0xBEEF), 0.9e18);

//     assertEq(token.totalSupply(), 1e18 - 0.9e18);
//     assertEq(token.balanceOf(address(0xBEEF)), 0.1e18);
//   }
// }
