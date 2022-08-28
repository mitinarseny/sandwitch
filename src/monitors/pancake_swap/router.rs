use web3::api::Eth;
use web3::types::Address;
use web3::Transport;

use super::contracts;
use super::factory::Factory;

pub struct Router<T: Transport> {
    contract: contracts::Router<T>,
    factory: Factory<T>,
}

impl<T: Transport> Router<T> {
    pub async fn new(eth: Eth<T>, address: Address) -> web3::contract::Result<Self> {
        let contract = contracts::Router::new(eth.clone(), address);
        let factory = Factory::new(eth, contract.factory().await?);
        Ok(Self { contract, factory })
    }

    pub fn address(&self) -> Address {
        self.contract.address()
    }

    pub fn factory(&self) -> &Factory<T> {
        &self.factory
    }
}
