use std::fmt::Display;

use futures::try_join;
use num::rational::Ratio;
use num::BigUint;
use web3::api::Eth;
use web3::types::Address;
use web3::Transport;

use super::contracts;
use super::factory::Factory;
use super::token::Token;

#[derive(Clone)]
pub struct Pair<T: Transport> {
    contract: contracts::Pair<T>,
    tokens: (Token<T>, Token<T>),
}

impl<T: Transport> Pair<T> {
    pub async fn new(
        eth: Eth<T>,
        factory: &Factory<T>,
        (mut t0, mut t1): (Address, Address),
    ) -> web3::contract::Result<Self> {
        if t0 > t1 {
            (t0, t1) = (t1, t0);
        }
        let (t0, t1, pair) = try_join!(
            Token::new(eth.clone(), t0),
            Token::new(eth.clone(), t1),
            factory.get_pair((t0, t1)),
        )?;
        Ok(Self {
            contract: contracts::Pair::new(eth, pair),
            tokens: (t0, t1),
        })
    }

    pub fn address(&self) -> Address {
        self.contract.address()
    }

    pub fn tokens(&self) -> &(Token<T>, Token<T>) {
        &self.tokens
    }

    pub async fn get_reserves(
        &self,
    ) -> web3::contract::Result<(Ratio<BigUint>, Ratio<BigUint>, u32)> {
        self.contract
            .get_reserves()
            .await
            .map(|(mut r0, mut r1, deadline)| {
                if self.inverse_order() {
                    (r0, r1) = (r1, r0);
                }
                (
                    self.tokens.0.as_decimals(r0),
                    self.tokens.1.as_decimals(r1),
                    deadline,
                )
            })
    }

    fn inverse_order(&self) -> bool {
        self.tokens.0 > self.tokens.1
    }
}

impl<T: Transport> Display for Pair<T> {
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
