use std::fmt::Display;
use std::ops::Deref;
use std::sync::Arc;

use ethers::prelude::{Address, ContractError, Middleware};
use futures::{future::try_join3, lock::Mutex};
use hex::ToHex;
use metrics::{register_counter, Counter};

use crate::cached::Aption;
use crate::contracts::{pancake_factory_v2::PancakeFactoryV2, pancake_pair::PancakePair};

use super::token::Token;

pub struct Pair<M: Middleware> {
    inner: PancakePair<M>,
    tokens: (Token<M>, Token<M>),
    inverse_order: bool,
    reserves: Mutex<Aption<(f64, f64, u32)>>,
    hit_times: Counter,
}

impl<M: Middleware> Deref for Pair<M> {
    type Target = PancakePair<M>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<M: Middleware> Pair<M> {
    pub async fn new(
        client: Arc<M>,
        factory: &PancakeFactoryV2<M>,
        (t0, t1): (Address, Address),
    ) -> Result<Self, ContractError<M>> {
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
                    ("token0", t0.address().encode_hex::<String>()),
                    ("token1", t0.address().encode_hex::<String>()),
                    ("token0_name", t0.name().to_string()),
                    ("token1_name", t1.name().to_string()),
                    ("pair", pair.encode_hex::<String>())
                ]
            ),
            tokens: (t0, t1),
            inner: PancakePair::new(pair, client),
            reserves: Mutex::new(None.into()),
        })
    }

    pub fn hit(&self) {
        self.hit_times.increment(1)
    }

    pub fn tokens(&self) -> &(Token<M>, Token<M>) {
        &self.tokens
    }

    async fn get_reserves(&self) -> Result<(f64, f64, u32), ContractError<M>> {
        let (mut r0, mut r1, deadline) = self.inner.get_reserves().call().await?;

        if self.inverse_order {
            (r0, r1) = (r1, r0);
        }

        let (t0, t1) = self.tokens();
        Ok((t0.as_decimals(r0), t1.as_decimals(r1), deadline))
    }

    pub async fn reserves(&self) -> Result<(f64, f64, u32), ContractError<M>> {
        self.reserves
            .lock()
            .await
            .get_or_try_insert_with(|| self.get_reserves())
            .await
            .map(|v| *v)
    }

    pub async fn on_block(&self) {
        self.reserves.lock().await.take();
    }
}

impl<M: Middleware> Display for Pair<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} -> {} -> {}",
            self.tokens.0,
            self.address(),
            self.tokens.1
        )
    }
}
