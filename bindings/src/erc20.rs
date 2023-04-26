use crate::prelude::*;

tracked_abigen!(ERC20, "contracts/out/ERC20.sol/ERC20.json");

impl EthTypedCall for ApproveCall {
    type Ok = maybe::OkOrNone;
    type Reverted = RawReverted;
}
