use hex::ToHex;
use std::fmt::Display;
use std::ops::Deref;
use std::sync::Arc;

use crate::contracts::pancake_factory_v2::PancakeFactoryV2;
use crate::contracts::pancake_pair::PancakePair;

use super::token::Token;
use ethers::prelude::{Address, ContractError, Middleware};
use futures::try_join;
use metrics::{register_counter, Counter};

#[derive(Clone)]
pub struct Pair<M: Middleware> {
    contract: PancakePair<M>,
    tokens: (Token<M>, Token<M>),
    inverse_order: bool,
}

impl<M: Middleware> Deref for Pair<M> {
    type Target = PancakePair<M>;

    fn deref(&self) -> &Self::Target {
        &self.contract
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
        let (t0, t1, pair) = {
            let pair = factory.get_pair(t0, t1);
            try_join!(
                Token::new(client.clone(), t0),
                Token::new(client.clone(), t1),
                pair.call(),
            )
        }?;
        Ok(Self {
            contract: PancakePair::new(pair, client),
            inverse_order: t0 > t1,
            tokens: (t0, t1),
        })
    }

    pub fn tokens(&self) -> &(Token<M>, Token<M>) {
        &self.tokens
    }

    pub async fn get_reserves(&self) -> Result<(f64, f64, u32), ContractError<M>> {
        self.contract
            .get_reserves()
            .call()
            .await
            .map(|(mut r0, mut r1, deadline)| {
                if self.inverse_order {
                    (r0, r1) = (r1, r0);
                }
                (
                    self.tokens.0.as_decimals(r0),
                    self.tokens.1.as_decimals(r1),
                    deadline,
                )
            })
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

pub struct CachedPair<M: Middleware> {
    inner: Pair<M>,
    reserves: Option<(f64, f64, u32)>,
    tx_count: Counter,
}

impl<M: Middleware> From<Pair<M>> for CachedPair<M> {
    fn from(p: Pair<M>) -> Self {
        let (t0, t1) = p.tokens();
        Self {
            reserves: None,
            tx_count: register_counter!(
                "sandwitch_pancake_swap_pair_hit_times",
                &[
                    ("token0", t0.address().encode_hex::<String>()),
                    ("token1", t0.address().encode_hex::<String>()),
                    ("token0_name", t0.name().to_string()),
                    ("token1_name", t1.name().to_string()),
                    ("pair", p.address().encode_hex::<String>())
                ]
            ),
            inner: p,
        }
    }
}

impl<M: Middleware> Deref for CachedPair<M> {
    type Target = Pair<M>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<M: Middleware> CachedPair<M> {
    pub async fn get_reserves(&mut self) -> Result<(f64, f64, u32), ContractError<M>> {
        Ok(match self.reserves {
            Some(v) => v,
            None => *self.reserves.insert(self.inner.get_reserves().await?),
        })
    }

    pub fn clear_cache(&mut self) {
        self.reserves.take();
    }

    pub fn hit(&self) {
        self.tx_count.increment(1);
    }
}
