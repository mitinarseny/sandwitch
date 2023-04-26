use crate::prelude::*;

tracked_abigen!(
    PancakeToaster,
    "contracts/out/PancakeToaster.sol/PancakeToaster.json"
);

impl EthTypedCall for FrontRunSwapExtCall {
    type Ok = FrontRunSwapExtReturn;
    type Reverted = PancakeToasterErrors;
}

impl EthTypedCall for BackRunSwapAllCall {
    type Ok = BackRunSwapAllReturn;
    type Reverted = PancakeToasterErrors;
}
