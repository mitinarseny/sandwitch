use std::sync::Arc;

use ethers::types::Address;
use sandwitch_contracts::pancake_swap;

pub struct PancakePair {

    contract: pancake_swap::router::PancakeRouter<M>,
    pub(super) tokens: [Address; 2],
}

impl<M> PancakePair<M> {
    pub(super) fn new(client: impl Into<Arc<M>>, address: Address, tokens: [Address; 2]) -> Self {
        Self {
            contract: pancake_swap::router::PancakeRouter::new(client, address),
            tokens,
        }
    }

    pub fn address(&self) -> Address {
        self.contract.address()
    }

    pub fn tokens(&self) -> &[Address; 2] {
        &self.tokens
    }
}
