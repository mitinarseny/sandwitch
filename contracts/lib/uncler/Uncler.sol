pragma solidity ^0.8.19;

abstract contract Uncler {
    error Uncled();

    // Uncle bandits: https://docs.flashbots.net/flashbots-protect/rpc/uncle-bandits
    modifier ensureParentBlock(bytes32 parentHash) {
        if (blockhash(block.number - 1) != parentHash) {
            revert Uncled();
        }

        _;
    }
}
