use std::fmt::Display;
use std::ops::Deref;
use std::sync::Arc;

use ethers::prelude::{Address, ContractError, Middleware};
use futures::lock::Mutex;

use crate::contracts::pancake_token::PancakeToken;

pub struct Token<M: Middleware> {
    contract: PancakeToken<M>,
    name: Mutex<Option<String>>,
    decimals: Mutex<Option<u8>>,
}

impl<M: Middleware> Deref for Token<M> {
    type Target = PancakeToken<M>;

    fn deref(&self) -> &Self::Target {
        &self.contract
    }
}

impl<M: Middleware> From<PancakeToken<M>> for Token<M> {
    fn from(c: PancakeToken<M>) -> Self {
        Self {
            contract: c,
            name: Mutex::new(None),
            decimals: Mutex::new(None),
        }
    }
}

impl<M: Middleware> Token<M> {
    pub fn new(client: Arc<M>, address: Address) -> Self {
        PancakeToken::new(address, client).into()
    }

    pub async fn name(&self) -> Result<String, ContractError<M>> {
        let mut name = self.name.lock().await;
        Ok(match &*name {
            Some(v) => v,
            None => name.insert(self.contract.name().call().await?),
        }
        .clone())
    }

    async fn decimals(&self) -> Result<u8, ContractError<M>> {
        let mut decimals = self.decimals.lock().await;
        Ok(match *decimals {
            Some(v) => v,
            None => *decimals.insert(self.contract.decimals().call().await?),
        })
    }

    pub async fn as_decimals(&self, v: impl Into<u128>) -> Result<f64, ContractError<M>> {
        Ok(v.into() as f64 / 10f64.powi(self.decimals().await? as i32))
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
