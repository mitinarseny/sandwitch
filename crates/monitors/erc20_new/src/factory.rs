use std::sync::Arc;

use ethers::{
    types::{Address, H256},
    utils::keccak256,
};
use hex_literal::hex;
use sandwitch_contracts::pancake_swap;

use super::pair::PancakePair;

pub struct PancakeFactory<M> {
    client: Arc<M>,
    contract: pancake_swap::factory::PancakeFactory<M>,
}

impl<M> PancakeFactory<M> {
    pub fn address(&self) -> Address {
        self.contract.address()
    }

    pub fn get_pair(&self, token_a: Address, token_b: Address) -> PancakePair<M> {
        // https://docs.uniswap.org/contracts/v2/guides/smart-contract-integration/getting-pair-addresses
        const PANCAKE_PAIR_INIT_CODE_HASH: H256 = H256(hex!(
            "96e8ac4277198ff8b6f785478aa9a39f403cb768dd02cbee326c3e7da348845f" // TODO
        ));

        let mut tokens = [token_a, token_b];
        tokens.sort();

        let address = H256(keccak256(
            [
                &[0xff, 0xff],
                self.address().0.as_slice(),
                keccak256(tokens.concat()).as_slice(),
                PANCAKE_PAIR_INIT_CODE_HASH.0.as_slice(),
            ]
            .concat(),
        ))
        .into();

        PancakePair::new(self.client.clone(), address, tokens)
    }
}
