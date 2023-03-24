pragma solidity ^0.8.19;

import {Owned} from "solmate/auth/Owned.sol";

import {MultiCall} from "multicall/MultiCall.sol";
import {Uncler} from "uncler/Uncler.sol";

contract OwnedMultiCall is MultiCall, Owned, Uncler {
    constructor() Owned(msg.sender) {}

    function multicall(bytes calldata commands, bytes[] calldata inputs)
        public
        payable
        override
        onlyOwner
        returns (bytes memory successes, bytes[] memory outputs)
    {
        (successes, outputs) = MultiCall.multicall(commands, inputs);
    }

    function multicall(bytes calldata commands, bytes[] calldata inputs, bytes32 requireParentBlockHash)
        external
        payable
        ensureParentBlock(requireParentBlockHash)
        returns (bytes memory successes, bytes[] memory outputs)
    {
        (successes, outputs) = multicall(commands, inputs);
    }
}
