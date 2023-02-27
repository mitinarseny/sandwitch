pragma solidity ^0.8.18;

import {IERC20} from "@openzeppelin/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/token/ERC20/utils/SafeERC20.sol";
import {Ownable} from "@openzeppelin/access/Ownable.sol";
import {SignedMath} from "@openzeppelin/utils/math/SignedMath.sol";

import {IPancakeFactory} from "@pancake_swap/interfaces/IPancakeFactory.sol";
import {IPancakePair} from "@pancake_swap/interfaces/IPancakePair.sol";

import {Toaster} from "@toaster/lib/Toaster.sol";

contract PancakeToaster is Ownable, Toaster {
  using SafeERC20 for IERC20;
  using SignedMath for uint256;

  IPancakeFactory public immutable factory;

  error SlippageExhausted();

  constructor(IPancakeFactory _factory) Ownable() {
    factory = _factory;
  }

  function frontRunSwapExactETHForTokens(
    address from,
    uint256 amountIn,
    uint256 amountOutMin,
    IERC20[] calldata path,
    uint256 indexIn,
    uint blockNumber
  )
    external
    ensureBlock(blockNumber)
    ensureBalance(from, amountIn)
    returns (
      uint256 ourAmountIn,
      uint256 ourAmountOut,
      uint256 newReserveIn,
      uint256 newReserveOut
  ) {
    return frontRunSwapExactIn(amountIn, amountOutMin, path, indexIn);
  }

  function frontRunSwapExactTokensForTokens(
    address from,
    uint256 amountIn,
    uint256 amountOutMin,
    IERC20[] calldata path,
    uint256 indexIn,
    uint blockNumber
  )
    external
    ensureBlock(blockNumber)
    ensureTokenBalance(path[0], from, amountIn)
    returns (
      uint256 ourAmountIn,
      uint256 ourAmountOut,
      uint256 newReserveIn,
      uint256 newReserveOut
  ) {
    return frontRunSwapExactIn(amountIn, amountOutMin, path, indexIn);
  }

  function frontRunSwapExactTokensForETH(
    address from,
    uint256 amountIn,
    uint256 amountOutMin,
    IERC20[] calldata path,
    uint256 indexIn,
    uint blockNumber
  )
    external
    ensureBlock(blockNumber)
    ensureTokenBalance(path[0], from, amountIn)
    returns (
      uint256 ourAmountIn,
      uint256 ourAmountOut,
      uint256 newReserveIn,
      uint256 newReserveOut
  ) {
    return frontRunSwapExactIn(amountIn, amountOutMin, path, indexIn);
  }

  function frontRunSwapETHForExactTokens(
    address from,
    uint256 amountOut,
    uint256 amountInMax,
    IERC20[] calldata path,
    uint256 indexIn,
    uint blockNumber
  )
    external
    ensureBlock(blockNumber)
    ensureBalance(from, amountInMax)
    returns (
      uint256 ourAmountIn,
      uint256 ourAmountOut,
      uint256 newReserveIn,
      uint256 newReserveOut
  ) {
    return frontRunSwapExactOut(amountOut, amountInMax, path, indexIn);
  }

  function frontRunSwapTokensForExactTokens(
    address from,
    uint256 amountOut,
    uint256 amountInMax,
    IERC20[] calldata path,
    uint256 indexIn,
    uint blockNumber
  )
    external
    ensureBlock(blockNumber)
    ensureTokenBalance(path[0], from, amountInMax)
    returns (
      uint256 ourAmountIn,
      uint256 ourAmountOut,
      uint256 newReserveIn,
      uint256 newReserveOut
  ) {
    return frontRunSwapExactOut(amountOut, amountInMax, path, indexIn);
  }

  function frontRunSwapTokensForExactETH(
    address from,
    uint256 amountOut,
    uint256 amountInMax,
    IERC20[] calldata path,
    uint256 indexIn,
    uint blockNumber
  ) external
    ensureBlock(blockNumber)
    ensureTokenBalance(path[0], from, amountInMax)
    returns (
      uint256 ourAmountIn,
      uint256 ourAmountOut,
      uint256 newReserveIn,
      uint256 newReserveOut
  ) {
    return frontRunSwapExactOut(amountOut, amountInMax, path, indexIn);
  }

  function frontRunSwapExactIn(
    uint256 amountIn,
    uint256 amountOutMin,
    IERC20[] memory path,
    uint256 indexIn,
  ) internal returns (
    uint256 ourAmountIn,
    uint256 ourAmountOut,
    uint256 newReserveIn,
    uint256 newReserveOut
  ) {
    (uint256 amountIn, uint256 amountOutMin) = PancakeBakerLibrary.reduceSwapExactIn(
      factory,
      amountIn,
      amountOutMin,
      path,
      indexIn
    );

    return frontRunSingleSwapExactIn(
      amountIn,
      amountOutMin,
      path[indexIn],
      path[indexIn + 1]
    );
  }

  function frontRunSingleSwapExactIn(
    uint256 amountIn,
    uint256 amountOutMin,
    IERC20 tokenIn,
    IERC20 tokenOut
  ) internal returns (
    uint256 ourAmountIn,
    uint256 ourAmountOut,
    uint256 newReserveIn,
    uint256 newReserveOut
  ) { // TODO: noreentrance
    uint256 available = tokenIn.balanceOf(msg.sender);
    // TODO: Kelly coefficient
    if (available == 0) {
      revert InsufficientTokenBalance(tokenIn, msg.sender);
    }

    (
      uint256 reserveIn,
      uint256 reserveOut,
      IPancakePair pair,
      bool invert
    ) = PancakeLibrary.getPairReserves(factory, tokenIn, tokenOut);

    uint256 ourAmountIn = PancakeBakerLibrary.getMaxFrontRunAmountIn(
      amountIn,
      amountOutMin,
      reserveIn,
      reserveOut,
    ).min(available);
    uint256 ourAmountOut = PancakeLibrary.getAmountOut(ourAmountIn, reserveIn, reserveOut);
    if (ourAmountOut == 0) {
      revert SlippageExhausted();
    }

    // TODO: check if to has allowance on tokenOut


    tokenIn.safeTransferFrom(msg.sender, address(pair), ourAmountIn);
    {
    (uint256 amount0Out, uint256 amount1Out) = invert ? (ourAmountOut, uint256(0)) : (uint256(0), ourAmountOut);
    pair.swap(amount0Out, amount1Out, address(this), new bytes(0));
    }
    // TODO: check that there is cycles or reuse of exploited pair

    (reserveIn, reserveOut) = pair.getReserves();
    if (invert) {
      (reserveIn, reserveOut) = (reserveOut, reserveIn);
    }

    return (ourAmountIn, ourAmountOut, reserveIn, reserveOut);
  }

  function backRunSwapAll(
    IERC20 tokenIn,
    IERC20 tokenOut,
  ) external onlyOwner returns (
    uint256 amountOut
  ) {
    uint256 amountIn = tokenIn.balanceOf(address(this));
    require(amountIn > 0, "PancakeToaster: INSUFFICIENT_BALANCE");

    (
      uint256 reserveIn,
      uint256 reserveOut,
      IPancakePair pair,
      bool invert
    ) = PancakeLibrary.getPairReserves(factory, tokenIn, tokenOut);
    uint256 amountOut = PancakeLibrary.getAmountOut(amountIn, reserveIn, reserveOut);

    tokenIn.safeTransfer(address(pair), amountIn);

    (uint256 amount0Out, uint256 amount1Out) = invert ? (uint256(0), amountOut) : (amountOut, uint256(0));

    pair.swap(amount0Out, amount1Out, msg.sender, new bytes(0));
  }
}
