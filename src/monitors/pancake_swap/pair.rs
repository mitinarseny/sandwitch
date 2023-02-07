use std::{fmt::Display, ops::Deref, sync::Arc};

use ethers::{
    abi::AbiEncode,
    prelude::{Address, ContractError},
    providers::{JsonRpcClient, Provider},
    types::H256,
};
use futures::future::try_join3;
use metrics::{register_counter, Counter};
use pancake_swap_contracts::{i_pancake_factory::IPancakeFactory, i_pancake_pair::IPancakePair};

use super::token::Token;
use crate::cached::CachedAtBlock;

pub struct Pair<P: JsonRpcClient> {
    inner: IPancakePair<Provider<P>>,
    tokens: (Token<P>, Token<P>),
    inverse_order: bool,
    reserves: CachedAtBlock<(f64, f64, u32)>,
    hit_times: Counter,
}

impl<P: JsonRpcClient> Deref for Pair<P> {
    type Target = IPancakePair<Provider<P>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<P: JsonRpcClient> Pair<P> {
    pub async fn new(
        client: impl Into<Arc<Provider<P>>>,
        factory: &IPancakeFactory<Provider<P>>,
        (t0, t1): (Address, Address),
    ) -> Result<Self, ContractError<Provider<P>>> {
        let client: Arc<_> = client.into();
        let (pair, t0, t1) = try_join3(
            factory.get_pair(t0, t1).call(),
            Token::new(client.clone(), t0),
            Token::new(client.clone(), t1),
        )
        .await?;
        Ok(Self {
            inverse_order: t0.address() > t1.address(),
            hit_times: register_counter!(
                "sandwitch_pancake_swap_pair_hit_times",
                &[
                    ("token0", t0.address().encode_hex()),
                    ("token1", t0.address().encode_hex()),
                    ("token0_name", t0.name().to_string()),
                    ("token1_name", t1.name().to_string()),
                    ("pair", pair.encode_hex())
                ]
            ),
            tokens: (t0, t1),
            inner: IPancakePair::new(pair, client),
            reserves: CachedAtBlock::default(),
        })
    }

    pub fn hit(&self) {
        self.hit_times.increment(1)
    }

    pub fn tokens(&self) -> &(Token<P>, Token<P>) {
        &self.tokens
    }

    async fn get_reserves_at(
        &self,
        block_hash: H256,
    ) -> Result<(f64, f64, u32), ContractError<Provider<P>>> {
        // TODO: use block id
        let (mut r0, mut r1, deadline) = self.inner.get_reserves().call().await?;

        if self.inverse_order {
            (r0, r1) = (r1, r0);
        }

        let (t0, t1) = self.tokens();
        Ok((t0.as_decimals(r0), t1.as_decimals(r1), deadline))
    }

    pub async fn reserves(
        &self,
        block_hash: H256,
    ) -> Result<(f64, f64, u32), ContractError<Provider<P>>> {
        self.reserves
            .get_at_or_try_insert_with(block_hash, |block_hash| self.get_reserves_at(*block_hash))
            .await
    }

    pub async fn on_block(&self) {
        self.reserves.flush().await
    }
}

impl<P: JsonRpcClient> Display for Pair<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} -> <{}> -> {}",
            self.tokens.0,
            self.address(),
            self.tokens.1
        )
    }
}
