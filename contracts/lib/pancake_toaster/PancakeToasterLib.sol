pragma solidity ^0.8.19;

import {ERC20} from "solmate/tokens/ERC20.sol";
import {FixedPointMathLib} from "solmate/utils/FixedPointMathLib.sol";

import {IPancakeFactory} from "pancake_swap/exchange-protocol/contracts/interfaces/IPancakeFactory.sol";
import {IPancakePair} from "pancake_swap/exchange-protocol/contracts/interfaces/IPancakePair.sol";

library PancakeToasterLib {
    using FixedPointMathLib for uint256;

    bytes32 internal constant PAIR_INIT_CODE_HASH =
        hex"a5934690703a592a07e841ca29d5e5c79b5e22ed4749057bb216dc31100be1c0";
    uint256 internal constant BIP = 1e4;
    uint256 internal constant FEE = 9975;

    error InsufficientInputAmount();
    error InsufficientOutputAmount();
    error InsufficientLiquidity();
    error InvalidPath();

    function sortTokens(ERC20 tokenA, ERC20 tokenB) internal pure returns (ERC20 token0, ERC20 token1) {
        require(tokenA != tokenB);
        (token0, token1) = tokenA < tokenB ? (tokenA, tokenB) : (tokenB, tokenA);
        require(address(token0) != address(0));
    }

    function pairForSorted(IPancakeFactory factory, ERC20 token0, ERC20 token1) internal pure returns (IPancakePair) {
        require(token0 < token1);
        return IPancakePair(
            address(
                uint160(
                    uint256(
                        keccak256(
                            abi.encodePacked(
                                hex"ff", factory, keccak256(abi.encodePacked(token0, token1)), PAIR_INIT_CODE_HASH
                            )
                        )
                    )
                )
            )
        );
    }

    function pairFor(IPancakeFactory factory, ERC20 tokenA, ERC20 tokenB)
        internal
        pure
        returns (IPancakePair pair, bool inverted)
    {
        (ERC20 token0, ERC20 token1) = sortTokens(tokenA, tokenB);
        return (pairForSorted(factory, token0, token1), tokenA != token0);
    }

    function getPairReserves(IPancakeFactory factory, ERC20 tokenA, ERC20 tokenB)
        internal
        view
        returns (uint256 reserveA, uint256 reserveB, IPancakePair pair, bool inverted)
    {
        (pair, inverted) = pairFor(factory, tokenA, tokenB);
        (reserveA, reserveB,) = pair.getReserves();
        if (inverted) (reserveA, reserveB) = (reserveB, reserveA);
        return (reserveA, reserveB, pair, inverted);
    }

    function getAmountOut(uint256 amountIn, uint256 reserveIn, uint256 reserveOut) internal pure returns (uint256) {
        if (amountIn == 0) {
            revert InsufficientInputAmount();
        }
        if (reserveIn == 0 || reserveOut == 0) {
            revert InsufficientLiquidity();
        }

        amountIn *= FEE;
        return (reserveOut * amountIn) / (reserveIn * BIP + amountIn);
    }

    function getAmountIn(uint256 amountOut, uint256 reserveIn, uint256 reserveOut) internal pure returns (uint256) {
        if (amountOut == 0) {
            revert InsufficientOutputAmount();
        }
        if (reserveIn == 0 || reserveOut == 0) {
            revert InsufficientLiquidity();
        }

        return (reserveIn * amountOut * BIP) / ((reserveOut - amountOut) * FEE) + 1;
    }

    function getAmountOut(IPancakeFactory factory, uint256 amountIn, ERC20[] memory path)
        internal
        view
        returns (uint256)
    {
        if (path.length < 2) {
            revert InvalidPath();
        }

        for (uint256 i; i < path.length - 1; ++i) {
            (uint256 reserveIn, uint256 reserveOut,,) = getPairReserves(factory, path[i], path[i + 1]);
            amountIn = getAmountOut(amountIn, reserveIn, reserveOut);
        }

        return amountIn;
    }

    function getAmountIn(IPancakeFactory factory, uint256 amountOut, ERC20[] memory path)
        internal
        view
        returns (uint256)
    {
        if (path.length < 2) {
            revert InvalidPath();
        }

        for (uint256 i = path.length - 1; i > 0; --i) {
            (uint256 reserveIn, uint256 reserveOut,,) = getPairReserves(factory, path[i - 1], path[i]);
            amountOut = getAmountIn(amountOut, reserveIn, reserveOut);
        }

        return amountOut;
    }

    function getMaxFrontRunAmountIn(uint256 amountIn, uint256 amountOut, uint256 reserveIn, uint256 reserveOut)
        internal
        pure
        returns (uint256)
    {
        return (
            (FEE * amountIn * (4 * BIP * reserveIn * reserveOut / amountOut + FEE * amountIn)).sqrt()
                - 2 * BIP * reserveIn - FEE * amountIn
        ) / (2 * FEE);
    }
}
