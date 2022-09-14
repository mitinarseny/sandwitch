use std::ops::Deref;
use std::sync::Arc;

use ethers::prelude::{Address, ContractError, Middleware};

use crate::contracts::pancake_factory_v2::PancakeFactoryV2;
use crate::contracts::pancake_router_v2::PancakeRouterV2;

#[derive(Clone)]
pub struct Router<M: Middleware> {
    contract: PancakeRouterV2<M>,
    factory: PancakeFactoryV2<M>,
}

impl<M: Middleware> Deref for Router<M> {
    type Target = PancakeRouterV2<M>;

    fn deref(&self) -> &Self::Target {
        &self.contract
    }
}

impl<M: Middleware> Router<M> {
    pub async fn new(client: Arc<M>, address: Address) -> Result<Self, ContractError<M>>
    where
        M: Clone,
    {
        let contract = PancakeRouterV2::new(address, client.clone());
        let factory = PancakeFactoryV2::new(contract.factory().call().await?, client);
        Ok(Self { contract, factory })
    }

    pub fn factory(&self) -> &PancakeFactoryV2<M> {
        &self.factory
    }
}
