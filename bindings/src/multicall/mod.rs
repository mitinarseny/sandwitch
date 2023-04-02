mod raw {
    use crate::prelude::*;

    tracked_abigen!(MultiCall, "contracts/out/MultiCall.sol/OwnedMultiCall.json");

    impl EthTypedCall for MulticallCall {
        type Ok = MulticallReturn;
        type Reverted = MultiCallErrors;
    }

    impl EthTypedCall for MulticallWithCommandsAndInputsCall {
        type Ok = MulticallWithCommandsAndInputsReturn;
        type Reverted = MultiCallErrors;
    }
}

mod calls;
mod contract;
mod errors;
mod multicall;

pub use self::{calls::*, contract::*, errors::*, multicall::*};
