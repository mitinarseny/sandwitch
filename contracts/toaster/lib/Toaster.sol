pragma solidity ^0.8.18;

import {IERC20} from "@openzeppelin/token/ERC20/IERC20.sol";

abstract contract Toaster {
  error Expired(uint number);
  error InsufficientBalance(address account);
  error InsufficientTokenBalance(IERC20 token, address account);

  modifier ensureBlock(uint number) {
    // TODO: https://docs.flashbots.net/flashbots-protect/rpc/uncle-bandits
    if (block.number != number) {
      revert Expired(number);
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
