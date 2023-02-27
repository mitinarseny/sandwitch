pragma solidity >= 0.8.17;

import {IERC20} from "@openzeppelin/token/ERC20/IERC20.sol";

import {PancakeLibrary} from "./PancakeLibrary.sol";

library PancakeBakerLibrary {
  error InvalidPath();

  function reduceSwapExactIn(
    IPancakeFactory factory,
    uint256 amountIn,
    uint256 amountOutMin,
    IERC20[] memory path,
    uint256 indexIn,
  ) internal view returns (
    uint256 amountIn,
    uint256 amountOutMin
  ) {
    if (indexIn + 1 >= path.length) {
      revert InvalidPath();
    }
    return (
      indexIn == 0 ? amountIn : PancakeLibrary.getAmountOut(factory, amountIn, path[:indexIn + 1]),
      indexIn + 2 == path.length ? amountOutMin : PancakeLibrary.getAmountIn(factory, amountOutMin, path[indexIn:])
    );
  }

  function reduceSwapExactOut(
    IPancakeFactory factory,
    uint256 amountInMax,
    uint256 amountOut,
    IERC20[] memory path,
    uint256 indexIn,
  ) internal view returns (
    uint256 amountInMax
    uint256 amountOut,
  ) {
    if (indexIn + 1 >= path.length) {
      revert InvalidPath();
    }
    return (
      indexIn == 0 ? amountInMax : PancakeLibrary.getAmountOut(factory, amountInMax, path[:indexIn + 1]),
      indexIn + 2 == path.length ? amountOut : PancakeLibrary.getAmountIn(factory, amountOut, path[indexIn:])
    );
  }

  function bakeMaxFrontAmountIn(
    uint256 amountIn,
    uint256 amountOutMin,
    uint256 reserveIn,
    uint256 reserveOut
  ) internal pure returns (uint256) {
    return reserveIn * reserveOut * amountIn / amountOutMin - reserveIn - amountIn;
  }

  function profit(
    uint256 amountIn,
    uint256 amountOutMin,
    uint256 reserveIn,
    uint256 reserveOut
  ) internal pure returns (uint256) {
    uint256 ourAmountIn = bakeMaxFrontAmountIn(amountIn, amountOutMin, reserveIn, reserveOut);
    
  }

  // function findAmountOutFor(
  //   address factory,
  //   uint256 amountIn,
  //   uint256 amountOutMin,
  //   address[] memory path
  // ) internal view returns (uint256 i, uint256) {
  //   require(path.length > 0, "BakerLibrary: INVALID_PATH");
  //   while (path[i] != item) {
  //     require(i < path.length - 1, "BakerLibrary: NOT_FOUND");
  //     (uint256 reserveIn, uint256 reserveOut) = PancakeLibrary.getReserves(factory, path[i], path[i + 1]);
  //     amountIn = PancakeLibrary.getAmountOut(amountIn, reserveIn, reserveOut);
  //     i++;
  //   }
  //   return (i, amountIn);
  // }
  //
  // function getAmountIn(
  //   address factory,
  //   uint256 amountOut,
  //   address[] memory path
  // ) internal view returns (uint256) {
  //   require(path.length > 0, "BakerLibrary: INVALID_PATH");
  //   for (uint256 i = path.length - 1; i > 0; i--) {
  //     (uint256 reserveIn, uint256 reserveOut) = PancakeLibrary.getReserves(factory, path[i - 1], path[i]);
  //     amountOut = PancakeLibrary.getAmountIn(amountOut, reserveIn, reserveOut);
  //   }
  //   return amountOut;
  // }
}
