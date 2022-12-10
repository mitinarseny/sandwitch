use std::ops::Deref;
use std::sync::Arc;

use ethers::prelude::{Address, ContractError, Middleware};
use ethers::providers::{JsonRpcClient, Provider};

use crate::contracts::pancake_factory_v2::PancakeFactoryV2;
use crate::contracts::pancake_router_v2::PancakeRouterV2;

#[derive(Clone)]
pub struct Router<P: JsonRpcClient> {
    contract: PancakeRouterV2<Provider<P>>,
    factory: PancakeFactoryV2<Provider<P>>,
}

impl<P: JsonRpcClient> Deref for Router<P> {
    type Target = PancakeRouterV2<Provider<P>>;

    fn deref(&self) -> &Self::Target {
        &self.contract
    }
}

impl<P: JsonRpcClient> Router<P> {
    pub async fn new(
        client: impl Into<Arc<Provider<P>>>,
        address: Address,
    ) -> Result<Self, ContractError<Provider<P>>> {
        let client = client.into();
        let contract = PancakeRouterV2::new(address, client.clone());
        let factory = PancakeFactoryV2::new(contract.factory().call().await?, client);
        Ok(Self { contract, factory })
    }

    pub fn factory(&self) -> &PancakeFactoryV2<Provider<P>> {
        &self.factory
    }
}
