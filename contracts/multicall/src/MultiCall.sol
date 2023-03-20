pragma solidity ^0.8.19;

bytes1 constant ALLOW_FAILURE = bytes1(uint8(1) << 7);
bytes1 constant WITH_VALUE    = bytes1(uint8(1) << 6);

contract MultiCall {
  enum Command {
    Group,
    Call,
    CallValue, // also used for Transfer
    GetBalanceOfThis,
    GetBalanceOfMsgSender,
    GetBalanceOfAddress,
    Create,
    CreateValue,
    Create2,
    Create2Value
  }

  struct Result {
    bool success;
    bytes output;
  }

  error LengthMismatch();
  error InvalidCommand();
  error Failed(uint256 index, bytes data);

  constructor() payable {}

  function multicall(bytes calldata commands, bytes[] calldata inputs) external payable returns (bytes memory successes, bytes[] memory outputs) {
    uint256 len = commands.length;
    if (inputs.length != len) revert LengthMismatch();
    successes = new bytes((len >> 3) + (len & 7 == 0 ? 0 : 1));
    outputs = new bytes[](len);

    bytes1 cmd;
    bool success;
    bytes memory output;

    for (uint256 i; i < len; ) {
        cmd = commands[i];
        output = outputs[i];
        (success, output) = dispatch(Command(uint8(cmd & ~ALLOW_FAILURE)), inputs[i]);
        if (success) {
            successes[i >> 3] |= bytes1(uint8(1)) << (i & 7);
        } else if (cmd & ALLOW_FAILURE == 0) {
            revert Failed(i, output);
        }
    }
  }

  function dispatch(Command cmd, bytes calldata inputs) internal returns (bool success, bytes memory output) {
    if (cmd == Command.Group) {
      return address(this).delegatecall(bytes.concat(this.multicall.selector, inputs));
    }

    if (cmd == Command.GetBalanceOfThis) {
      return (true, abi.encodePacked(address(this).balance));
    }

    if (cmd == Command.GetBalanceOfMsgSender) {
      return (true, abi.encodePacked(msg.sender.balance));
    }

    if (cmd == Command.GetBalanceOfAddress) {
      address target = address(bytes20(inputs));
      return (true, abi.encodePacked(target.balance));
    }

    uint256 value;
    if (cmd == Command.CallValue || cmd == Command.CreateValue || cmd == Command.Create2Value) {
      value = uint256(bytes32(inputs));
      inputs = inputs[32:];
    }

    if (cmd == Command.Call || cmd == Command.CallValue) {
      address target = address(bytes20(inputs));
      return target.call{value: value}(inputs[20:]);
    }

    address addr;
    if (cmd == Command.Create || cmd == Command.CreateValue) {
      assembly ("memory-safe") {
        let p := mload(0x40)
        mstore(0x40, add(p, inputs.length))
        calldatacopy(p, inputs.offset, inputs.length)

        addr := create(value, p, inputs.length)
      }
    } else if (cmd == Command.Create2 || cmd == Command.Create2Value) {
      uint256 salt = uint256(bytes32(inputs));
      inputs = inputs[32:];

      assembly ("memory-safe") {
        let p := mload(0x40)
        mstore(0x40, add(p, inputs.length))
        calldatacopy(p, inputs.offset, inputs.length)

        addr := create2(value, p, inputs.length, salt)
      }
    } else {
      revert InvalidCommand();
    }
    if (success = (addr != address(0))) {
      output = abi.encodePacked(addr);
    }
  }
}
