pragma solidity ^0.8.19;

abstract contract MultiCall {
    bytes1 constant ALLOW_FAILURE = bytes1(uint8(1) << 7);

    // up to 127 max
    enum Command {
        Group, // 0
        Call, // 1
        CallValue, // 2, also used for Transfer
        GetBalanceOfThis, // 3
        GetBalanceOfMsgSender, // 4
        GetBalanceOfAddress, // 5
        Create, // 6
        CreateValue, // 7
        Create2, // 8
        Create2Value // 9
    }

    error LengthMismatch();
    error InvalidCommand();
    error Reverted(uint256 index, bytes data);

    function multicall(bytes calldata commands, bytes[] calldata inputs)
        public
        payable
        virtual
        returns (bytes memory successes, bytes[] memory outputs)
    {
        uint256 len = commands.length;
        if (inputs.length != len) revert LengthMismatch();
        successes = new bytes((len + 7) >> 3);
        outputs = new bytes[](len);

        bytes1 cmd;
        bool success;
        bytes memory output;

        for (uint256 i; i < len;) {
            cmd = commands[i];
            output = outputs[i];
            (success, output) = dispatch(Command(uint8(cmd & ~ALLOW_FAILURE)), inputs[i]);
            if (success) {
                successes[i >> 3] |= bytes1(uint8(0x8)) >> (i & 7);
            } else if (cmd & ALLOW_FAILURE == 0) {
                revert Reverted(i, output);
            }
        }
    }

    function dispatch(Command cmd, bytes calldata input) internal returns (bool success, bytes memory output) {
        if (cmd == Command.Group) {
            return address(this).delegatecall(bytes.concat(this.multicall.selector, input));
        }

        if (cmd == Command.GetBalanceOfThis) {
            return (true, abi.encodePacked(address(this).balance));
        }

        if (cmd == Command.GetBalanceOfMsgSender) {
            return (true, abi.encodePacked(msg.sender.balance));
        }

        if (cmd == Command.GetBalanceOfAddress) {
            address target = address(bytes20(input));
            return (true, abi.encodePacked(target.balance));
        }

        uint256 value;
        if (cmd == Command.CallValue || cmd == Command.CreateValue || cmd == Command.Create2Value) {
            value = uint256(bytes32(input));
            input = input[32:];
        }

        if (cmd == Command.Call || cmd == Command.CallValue) {
            address target = address(bytes20(input));
            return target.call{value: value}(input[20:]);
        }

        address addr;
        if (cmd == Command.Create || cmd == Command.CreateValue) {
            assembly ("memory-safe") {
                let p := mload(0x40)
                mstore(0x40, add(p, input.length))
                calldatacopy(p, input.offset, input.length)

                addr := create(value, p, input.length)
            }
        } else if (cmd == Command.Create2 || cmd == Command.Create2Value) {
            uint256 salt = uint256(bytes32(input));
            input = input[32:];

            assembly ("memory-safe") {
                let p := mload(0x40)
                mstore(0x40, add(p, input.length))
                calldatacopy(p, input.offset, input.length)

                addr := create2(value, p, input.length, salt)
            }
        } else {
            revert InvalidCommand();
        }
        if (success = (addr != address(0))) {
            output = abi.encodePacked(addr);
        }
    }
}
