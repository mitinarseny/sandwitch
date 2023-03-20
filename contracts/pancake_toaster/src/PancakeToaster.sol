pragma solidity ^0.8.18;

import {IERC20} from "@openzeppelin/token/ERC20/IERC20.sol";
import {IERC20Metadata} from "@openzeppelin/token/ERC20/extensions/IERC20Metadata.sol";
import {SafeERC20} from "@openzeppelin/token/ERC20/utils/SafeERC20.sol";
import {Ownable} from "@openzeppelin/access/Ownable.sol";
import {Math} from "@openzeppelin/utils/math/Math.sol";

import {IPancakeFactory} from "@pancake_swap/interfaces/IPancakeFactory.sol";
import {IPancakePair} from "@pancake_swap/interfaces/IPancakePair.sol";

import {Toaster} from "@toaster/lib/Toaster.sol";
import {PancakeLibrary} from "../lib/PancakeLibrary.sol";

contract PancakeToaster is Ownable, Toaster {
  using Math for uint256;
  using SafeERC20 for IERC20;
  using PancakeLibrary for IPancakeFactory;

  IPancakeFactory public immutable factory;

  error SlippageExhausted();

  constructor(IPancakeFactory _factory) Ownable() {
    factory = _factory;
  }

  // for calculating profit off-chain
  function frontRunSwapExt(
    address from,
    uint256 amountIn,
    uint256 amountOut,
    bool ETHIn,
    IERC20[] calldata path,
    uint256 indexIn,
    bytes32 parentBlockHash
  ) external returns (
    uint256 ourAmountIn,
    uint256 ourAmountOut,
    uint256 newReserveIn,
    uint256 newReserveOut
  ) {
    (ourAmountIn, ourAmountOut) = frontRunSwap(
      from,
      amountIn,
      amountOut,
      ETHIn,
      path,
      indexIn,
      parentBlockHash
    );

    (newReserveIn, newReserveOut, , ) = factory.getPairReserves(
      path[indexIn], path[indexIn + 1]);
    return (ourAmountIn, ourAmountOut, newReserveIn, newReserveOut);
  }

  function frontRunSwap(
    address from,
    uint256 amountIn,
    uint256 amountOut,
    bool ETHIn,
    IERC20[] calldata path,
    uint256 indexIn,
    bytes32 parentBlockHash
  ) public ensureParentBlock(parentBlockHash) returns (
    uint256 ourAmountIn,
    uint256 ourAmountOut
  ) {
    if (indexIn + 1 >= path.length) {
      revert PancakeLibrary.InvalidPath();
    }

    if ((ETHIn ? from.balance : path[0].balanceOf(from)) < amountIn) {
      // TODO: check user approval on first token for router
      revert InsufficientBalance(from);
    }

    if (indexIn > 0) {
      amountIn = factory.getAmountOut(amountIn, path[:indexIn + 1]);
    }
    if (indexIn + 2 < path.length) {
      amountOut = factory.getAmountIn(amountOut, path[indexIn + 1:]);
    }

    return frontRunSingleSwap(amountIn, amountOut, path[indexIn], path[indexIn + 1]);
  }

  function frontRunSingleSwap(
    uint256 amountIn,
    uint256 amountOut,
    IERC20 tokenIn,
    IERC20 tokenOut
  ) internal returns (
    uint256 ourAmountIn,
    uint256 ourAmountOut
  ) { // TODO: noreentrance
    uint256 available = tokenIn.balanceOf(msg.sender);
    // TODO: Kelly coefficient
    if (available == 0) {
      revert InsufficientBalance(msg.sender);
    }

    (
      uint256 reserveIn,
      uint256 reserveOut,
      IPancakePair pair,
      bool inverted
    ) = factory.getPairReserves(tokenIn, tokenOut);

    ourAmountIn = PancakeLibrary.getMaxFrontRunAmountIn(
      amountIn,
      amountOut,
      reserveIn,
      reserveOut
    ).min(available);
    ourAmountOut = PancakeLibrary.getAmountOut(ourAmountIn, reserveIn, reserveOut);
    if (ourAmountOut == 0) {
      revert SlippageExhausted();
    }

    tokenIn.safeTransferFrom(msg.sender, address(pair), ourAmountIn);
    {
    (uint256 amount0Out, uint256 amount1Out) = inverted ? (ourAmountOut, uint256(0)) : (uint256(0), ourAmountOut);
    pair.swap(amount0Out, amount1Out, address(this), new bytes(0));
    }

    // TODO: check that there is cycles or reuse of exploited pair
    return (ourAmountIn, ourAmountOut);
  }

  function backRunSwapAll(
    IERC20 tokenIn,
    IERC20 tokenOut
  ) external onlyOwner returns (uint256 amountOut) {
    uint256 amountIn = tokenIn.balanceOf(address(this));
    if (amountIn == 0) {
      revert InsufficientBalance(address(this));
    }

    (
      uint256 reserveIn,
      uint256 reserveOut,
      IPancakePair pair,
      bool inverted
    ) = factory.getPairReserves(tokenIn, tokenOut);
    amountOut = PancakeLibrary.getAmountOut(amountIn, reserveIn, reserveOut);

    tokenIn.safeTransfer(address(pair), amountIn);
    {
    (uint256 amount0Out, uint256 amount1Out) = inverted ? (amountOut, uint256(0)) : (uint256(0), amountOut);
    pair.swap(amount0Out, amount1Out, msg.sender, new bytes(0));
    }

    return amountOut;
  }
}
