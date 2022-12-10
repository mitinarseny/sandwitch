use std::{fmt::Display, ops::Deref, sync::Arc};

use ethers::providers::{JsonRpcClient, Provider};
use ethers::{contract::ContractError, providers::Middleware, types::Address};
use futures::future::try_join;

use crate::contracts::pancake_token::PancakeToken;

pub struct Token<P: JsonRpcClient> {
    contract: PancakeToken<Provider<P>>,
    name: String,
    decimals: u8,
}

impl<P: JsonRpcClient> Deref for Token<P> {
    type Target = PancakeToken<Provider<P>>;

    fn deref(&self) -> &Self::Target {
        &self.contract
    }
}

impl<P: JsonRpcClient> Token<P> {
    pub async fn new(
        client: impl Into<Arc<Provider<P>>>,
        address: Address,
    ) -> Result<Self, ContractError<Provider<P>>> {
        let contract = PancakeToken::new(address, client.into());
        let (name, decimals) = try_join(contract.name().call(), contract.decimals().call()).await?;
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
        v.into() as f64 / (10u128.pow(self.decimals as u32) as f64)
    }

    pub fn from_decimals(&self, v: f64) -> u128 {
        (v * (10u128.pow(self.decimals as u32) as f64)) as u128
    }
}

impl<P: JsonRpcClient> PartialEq for Token<P> {
    fn eq(&self, other: &Self) -> bool {
        self.address().eq(&other.address())
    }
}

impl<P: JsonRpcClient> PartialOrd for Token<P> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.address().partial_cmp(&other.address())
    }
}

impl<P: JsonRpcClient> Display for Token<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.address())
    }
}
