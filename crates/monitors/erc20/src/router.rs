use std::ops::Deref;
use std::sync::Arc;

use ethers::prelude::{Address, ContractError};
use ethers::providers::{JsonRpcClient, Provider};

use contracts::pancake_swap::{
    i_pancake_factory::IPancakeFactory, i_pancake_router_02::IPancakeRouter02,
};

#[derive(Clone)]
pub struct Router<P: JsonRpcClient> {
    contract: IPancakeRouter02<Provider<P>>,
    factory: IPancakeFactory<Provider<P>>,
}

impl<P: JsonRpcClient> Deref for Router<P> {
    type Target = IPancakeRouter02<Provider<P>>;

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
        let contract = IPancakeRouter02::new(address, client.clone());
        let factory = IPancakeFactory::new(contract.factory().call().await?, client);
        Ok(Self { contract, factory })
    }

    pub fn factory(&self) -> &IPancakeFactory<Provider<P>> {
        &self.factory
    }
}
