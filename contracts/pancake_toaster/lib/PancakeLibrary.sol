pragma solidity ^0.8.18;

import {IERC20} from "@openzeppelin/token/ERC20/IERC20.sol";

import {IPancakeFactory} from "@pancake_swap/interfaces/IPancakeFactory.sol";
import {IPancakePair} from "@pancake_swap/interfaces/IPancakePair.sol";
import {PancakePair} from "@pancake_swap/PancakePair.sol";

library PancakeLibrary {
  bytes32 internal constant PAIR_INIT_CODE_HASH = keccak256(abi.encodePacked(type(PancakePair).creationCode));
  uint256 internal constant BIP = 1e6;
  uint256 internal constant FEE = 9975;

  error InsufficientInputAmount();
  error InsufficientOutputAmount();
  error InsufficientLiquidity();
  error InvalidPath();

  function sortTokens(
    IERC20 tokenA,
    IERC20 tokenB
  ) internal pure returns (address token0, address token1) {
    require(tokenA != tokenB);
    (token0, token1) = tokenA < tokenB ? (tokenA, tokenB) : (tokenB, tokenA);
    require(token0 != address(0));
  }

  function pairForSorted(
    IPancakeFactory factory,
    IERC20 token0,
    IERC20 token1
  ) internal pure returns (address) {
    require(token0 < token1);
    address(uint160(uint256(keccak256(abi.encodePacked(
      hex"ff",
      factory,
      keccak256(abi.encodePacked(token0, token1)),
      PAIR_INIT_CODE_HASH,
    )))))
  }

  function pairFor(
    IPancakeFactory factory,
    IERC20 tokenA,
    IERC20 tokenB
  ) internal pure returns (IPancakePair pair, bool inverted) {
    (address token0, address token1) = sortTokens(tokenA, tokenB);
    return (pairForSorted(factory, token0, token1), tokenA != token0);
  }

  function getPairReserves(
    IPancakeFactory factory,
    IERC20 tokenA,
    IERC20 tokenB
  ) internal view returns (
    uint256 reserveA,
    uint256 reserveB,
    IPancakePair pair,
    bool inverted
  ) {
    (pair, inverted) = pairFor(factory, tokenA, tokenB);
    (reserveA, reserveB) = pair.getReserves();
    if (inverted) (reserveA, reserveB) = (reserveB, reserveA);
    return (reserveA, reserveB, pair, inverted);
  }

  function getAmountOut(
    uint256 amountIn,
    uint256 reserveIn,
    uint256 reserveOut
  ) internal pure returns (uint256 amountOut) {
    if (amountIn == 0) {
      revert InsufficientInputAmount();
    }
    if (reserveIn == 0 || reserveOut == 0) {
      revert InsufficientLiquidity();
    }

    amountIn *= FEE;
    return (reserveOut * amountIn) / (reserveIn * BIP + amountIn);
  }

  function getAmountIn(
    uint256 amountOut,
    uint256 reserveIn,
    uint256 reserveOut
  ) internal pure returns (uint256 amountIn) {
    if (amountOut == 0) {
      revert InsufficientOutputAmount();
    }
    if (reserveIn == 0 || reserveOut == 0) {
      revert InsufficientLiquidity();
    }

    return (reserveIn * amountOut * BIP) / ((reserveOut - amountOut) * FEE) + 1;
  }

  function getAmountOut(
    IPancakeFactory factory,
    uint256 amountIn,
    IERC20[] memory path
  ) internal view returns (uint256) {
    if (path.length < 2) {
      revert InvalidPath();
    }

    for (uint256 i; i < path.length - 1; ++i) {
      (uint256 reserveIn, uint256 reserveOut, ) = getPairReserves(factory, path[i], path[i + 1]);
      amountIn = getAmountOut(amountIn, reserveIn, reserveOut);
    }

    return amountIn;
  }

  function getAmountIn(
    IPancakeFactory factory,
    uint256 amountOut,
    IERC20[] memory path
  ) internal view returns (uint256) {
    if (path.length < 2) {
      revert InvalidPath();
    }

    for (uint256 i = path.length - 1; i > 0; --i) {
      (uint256 reserveIn, uint256 reserveOut, ) = getPairReserves(factory, path[i - 1], path[i]);
      amountOut = getAmountIn(amountOut, reserveIn, reserveOut);
    }

    return amountOut;
  }
}
