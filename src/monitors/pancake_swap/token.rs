use std::fmt::Display;
use std::ops::Deref;
use std::sync::Arc;

use ethers::prelude::{Address, ContractError, Middleware};
use futures::future::try_join;

use crate::contracts::pancake_token::PancakeToken;

pub struct Token<M: Middleware> {
    contract: PancakeToken<M>,
    name: String,
    decimals: u8,
}

impl<M: Middleware> Deref for Token<M> {
    type Target = PancakeToken<M>;

    fn deref(&self) -> &Self::Target {
        &self.contract
    }
}

impl<M: Middleware> Token<M> {
    pub async fn new(client: Arc<M>, address: Address) -> Result<Self, ContractError<M>> {
        let contract = PancakeToken::new(address, client);
        let (name, decimals) =
            try_join(contract.name().call(), contract.decimals().call()).await?;
        Ok(Self {
            contract,
            name,
            decimals,
        })
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn as_decimals(&self, v: impl Into<u128>) -> f64 {
        v.into() as f64 / 10f64.powi(self.decimals as i32)
    }
}

impl<M: Middleware> PartialEq for Token<M> {
    fn eq(&self, other: &Self) -> bool {
        self.address().eq(&other.address())
    }
}

impl<M: Middleware> PartialOrd for Token<M> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.address().partial_cmp(&other.address())
    }
}

impl<M: Middleware> Display for Token<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.address())
    }
}
