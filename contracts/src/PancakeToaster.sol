pragma solidity ^0.8.19;

import {ERC20} from "solmate/tokens/ERC20.sol";
import {SafeTransferLib} from "solmate/utils/SafeTransferLib.sol";
import {Owned} from "solmate/auth/Owned.sol";

import {IPancakeFactory} from "pancake_swap/exchange-protocol/contracts/interfaces/IPancakeFactory.sol";
import {IPancakePair} from "pancake_swap/exchange-protocol/contracts/interfaces/IPancakePair.sol";
import {IPancakeRouter02} from "pancake_swap/exchange-protocol/contracts/interfaces/IPancakeRouter02.sol";

import {PancakeToasterLib} from "pancake_toaster/PancakeToasterLib.sol";

contract PancakeToaster is Owned {
    using SafeTransferLib for ERC20;
    using PancakeToasterLib for IPancakeFactory;

    IPancakeFactory public immutable factory;

    error InsufficientBalance(address);
    error SlippageExhausted();

    constructor(IPancakeFactory _factory) Owned(msg.sender) {
        factory = _factory;
    }

    // for calculating profit off-chain
    function frontRunSwapExt(
        address from,
        uint256 amountIn,
        uint256 amountOut,
        bool ETHIn,
        ERC20[] calldata path,
        uint256 indexIn
    ) external returns (uint256 ourAmountIn, uint256 ourAmountOut, uint256 newReserveIn, uint256 newReserveOut) {
        (ourAmountIn, ourAmountOut) = frontRunSwap(from, amountIn, amountOut, ETHIn, path, indexIn);

        (newReserveIn, newReserveOut,,) = factory.getPairReserves(path[indexIn], path[indexIn + 1]);
        return (ourAmountIn, ourAmountOut, newReserveIn, newReserveOut);
    }

    function frontRunSwap(
        address from,
        uint256 amountIn,
        uint256 amountOut,
        bool ETHIn,
        ERC20[] calldata path,
        uint256 indexIn
    ) public returns (uint256 ourAmountIn, uint256 ourAmountOut) {
        if (indexIn + 1 >= path.length) {
            revert PancakeToasterLib.InvalidPath();
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

    function frontRunSingleSwap(uint256 amountIn, uint256 amountOut, ERC20 tokenIn, ERC20 tokenOut)
        internal
        returns (uint256 ourAmountIn, uint256 ourAmountOut)
    {
        // TODO: noreentrance
        uint256 available = tokenIn.balanceOf(msg.sender);
        // TODO: Kelly coefficient
        if (available == 0) {
            revert InsufficientBalance(msg.sender);
        }

        (uint256 reserveIn, uint256 reserveOut, IPancakePair pair, bool inverted) =
            factory.getPairReserves(tokenIn, tokenOut);

        ourAmountIn = PancakeToasterLib.getMaxFrontRunAmountIn(amountIn, amountOut, reserveIn, reserveOut);
        ourAmountIn = ourAmountIn < available ? ourAmountIn : available;
        ourAmountOut = PancakeToasterLib.getAmountOut(ourAmountIn, reserveIn, reserveOut);
        if (ourAmountOut == 0) {
            revert SlippageExhausted();
        }

        tokenIn.safeTransferFrom(msg.sender, address(pair), ourAmountIn);
        {
            (uint256 amount0Out, uint256 amount1Out) =
                inverted ? (ourAmountOut, uint256(0)) : (uint256(0), ourAmountOut);
            pair.swap(amount0Out, amount1Out, address(this), new bytes(0));
        }

        // TODO: check that there is cycles or reuse of exploited pair
        return (ourAmountIn, ourAmountOut);
    }

    function backRunSwapAll(ERC20 tokenIn, ERC20 tokenOut) external onlyOwner returns (uint256 amountOut) {
        uint256 amountIn = tokenIn.balanceOf(address(this));
        if (amountIn == 0) {
            revert InsufficientBalance(address(this));
        }

        (uint256 reserveIn, uint256 reserveOut, IPancakePair pair, bool inverted) =
            factory.getPairReserves(tokenIn, tokenOut);
        amountOut = PancakeToasterLib.getAmountOut(amountIn, reserveIn, reserveOut);

        tokenIn.safeTransfer(address(pair), amountIn);
        {
            (uint256 amount0Out, uint256 amount1Out) = inverted ? (amountOut, uint256(0)) : (uint256(0), amountOut);
            pair.swap(amount0Out, amount1Out, msg.sender, new bytes(0));
        }

        return amountOut;
    }
}
