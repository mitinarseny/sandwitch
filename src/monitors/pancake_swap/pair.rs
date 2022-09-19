use std::fmt::Display;
use std::ops::Deref;
use std::sync::Arc;

use ethers::prelude::{Address, ContractError, Middleware};
use futures::{future::try_join, lock::Mutex, TryFutureExt};
use hex::ToHex;
use metrics::{register_counter, Counter};

use crate::contracts::{pancake_factory_v2::PancakeFactoryV2, pancake_pair::PancakePair};

use super::token::Token;

pub struct Pair<M: Middleware> {
    inner: PancakePair<M>,
    tokens: (Token<M>, Token<M>),
    inverse_order: bool,
    reserves: Mutex<Option<(f64, f64, u32)>>,
    hit_times: Mutex<Option<Counter>>,
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
    ) -> Result<Self, ContractError<M>>
    where
        M: Clone,
    {
        Ok(Self {
            tokens: (
                Token::new(client.clone(), t0),
                Token::new(client.clone(), t1),
            ),
            inner: PancakePair::new(factory.get_pair(t0, t1).call().await?, client),
            inverse_order: t0 > t1,
            reserves: Mutex::new(None),
            hit_times: Mutex::new(None),
        })
    }

    async fn hit_times(&self) -> Result<Counter, ContractError<M>> {
        let mut hit_times = self.hit_times.lock().await;
        Ok(match &*hit_times {
            Some(c) => c,
            None => {
                let (t0, t1) = self.tokens();
                hit_times.insert(register_counter!(
                    "sandwitch_pancake_swap_pair_hit_times",
                    &[
                        ("token0", t0.address().encode_hex::<String>()),
                        ("token1", t0.address().encode_hex::<String>()),
                        ("token0_name", t0.name().await?.to_string()),
                        ("token1_name", t1.name().await?.to_string()),
                        ("pair", self.inner.address().encode_hex::<String>())
                    ]
                ))
            }
        }
        .clone())
    }

    pub async fn hit(&self) -> Result<(), ContractError<M>> {
        self.hit_times().await?.increment(1);
        Ok(())
    }

    pub fn tokens(&self) -> &(Token<M>, Token<M>) {
        &self.tokens
    }

    async fn get_reserves(&self) -> Result<(f64, f64, u32), ContractError<M>> {
        self.inner
            .get_reserves()
            .call()
            .and_then(|(mut r0, mut r1, deadline)| async move {
                if self.inverse_order {
                    (r0, r1) = (r1, r0);
                }

                let (t0, t1) = self.tokens();
                let (r0, r1) = try_join(t0.as_decimals(r0), t1.as_decimals(r1)).await?;
                Ok((r0, r1, deadline))
            })
            .await
    }

    pub async fn reserves(&self) -> Result<(f64, f64, u32), ContractError<M>> {
        let mut reserves = self.reserves.lock().await;
        Ok(match *reserves {
            Some(v) => v,
            None => *reserves.insert(self.get_reserves().await?),
        })
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
