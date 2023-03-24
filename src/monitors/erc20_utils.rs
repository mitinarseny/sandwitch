use ethers::prelude::builders::ContractCall;
use ethers::providers::{Middleware, JsonRpcClient, Provider};
use ethers::types::{Address, Bytes, U256};

use crate::cached::CachedAtBlock;

pub(crate) trait Pair<M: Middleware> {
    fn get_reserves(&self) -> ContractCall<M, U256>;
    fn swap(
        &self,
        amount_0_out: U256,
        amount_1_out: U256,
        to: Address,
        data: Bytes,
    ) -> ContractCall<M, ()>;
}

pub(crate) struct CachedPair<C: JsonRpcClient, P: Pair<Provider<C>>> {
    inner: P,
    reserves: CachedAtBlock<(U256, U256)>,
}
