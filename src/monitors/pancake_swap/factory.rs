use super::contracts;
use web3::api::Eth;
use web3::types::Address;
use web3::Transport;

#[derive(Clone)]
pub struct Factory<T: Transport> {
    contract: contracts::Factory<T>,
}

impl<T: Transport> Factory<T> {
    pub fn new(eth: Eth<T>, address: Address) -> Self {
        Self {
            contract: contracts::Factory::new(eth, address),
        }
    }

    pub async fn get_pair(&self, (t0, t1): (Address, Address)) -> anyhow::Result<Address> {
        self.contract.get_pair((t0, t1)).await
    }
}
