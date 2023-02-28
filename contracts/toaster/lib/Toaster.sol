pragma solidity ^0.8.18;

import {IERC20} from "@openzeppelin/token/ERC20/IERC20.sol";

abstract contract Toaster {
  error Uncled();
  error InsufficientBalance(address account);
  error InsufficientTokenBalance(IERC20 token, address account);

  // Uncle bandits: https://docs.flashbots.net/flashbots-protect/rpc/uncle-bandits
  modifier ensureParentBlock(bytes32 parentHash) {
    if (blockhash(block.number - 1) != parentHash) {
      revert Uncled();
    }

    _;
  }

  modifier ensureBalance(address account, uint256 amount) {
    if (account.balance < amount) {
      revert InsufficientBalance(account);
    }

    _;
  }

  modifier ensureTokenBalance(IERC20 token, address account, uint256 amount) {
    if (token.balanceOf(account) < amount) {
      revert InsufficientTokenBalance(token, account);
    }

    _;
  }
}
