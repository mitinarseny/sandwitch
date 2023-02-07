pragma solidity >= 0.8.17;

import "pancake-smart-contracts/projects/exchange-protocol/contracts/libraries/PancakeLibrary.sol";

library BakerLibrary {
  function findAmountOutFor(
    address factory,
    uint256 amountIn,
    address[] memory path,
    address item,
  ) internal view returns (uint256 i, uint256) {
    require(path.length > 0, "BakerLibrary: INVALID_PATH");
    while (path[i] != item) {
      require(i < path.length - 1, "BakerLibrary: NOT_FOUND");
      (uint256 reserveIn, uint256 reserveOut) = PancakeLibrary.getReserves(factory, path[i], path[i + 1]);
      amountIn = PancakeLibrary.getAmountOut(amountIn, reserveIn, reserveOut);
      i++;
    }
    return (i, amountIn)
  }

  function getAmountIn(
    address factory,
    uint256 amountOut,
    address[] memory path,
  ) internal view returns (uint256) {
    require(path.length > 0, "BakerLibrary: INVALID_PATH");
    for (uint256 i = path.length - 1; i > 0; i--) {
      (uint256 reserveIn, uint256 reserveOut) = PancakeLibrary.getReserves(factory, path[i - 1], path[i]);
      amountOut = PancakeLibrary.getAmountIn(amountOut, reserveIn, reserveOut);
    }
    return amountOut;
  }
}
