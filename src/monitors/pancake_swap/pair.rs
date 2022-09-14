use std::fmt::Display;
use std::ops::Deref;
use std::sync::Arc;

use crate::contracts::pancake_factory_v2::PancakeFactoryV2;
use crate::contracts::pancake_pair::PancakePair;

use super::token::Token;
use ethers::prelude::{Address, ContractError, Middleware};
use futures::try_join;

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
