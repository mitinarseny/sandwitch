use std::collections::HashMap;

use ethers::types::{Address, U256};

// use super::pair::PancakePair;

#[derive(Debug)]
pub struct Swap {
    pub from: Address,
    pub amount_in: U256,
    pub amount_out: U256,
    pub eth_in: bool,
    pub path: Vec<Address>,
    // pub pairs: HashMap<Address, PancakePair<M>>,
}

impl Swap {
    pub fn pairs(&self) -> impl Iterator<Item = Address> {
        None.into_iter()
        // todo!()
    }
}
