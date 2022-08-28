use std::collections::BTreeMap;

use futures::TryFutureExt;
use lazy_static::lazy_static;
use web3::api::Eth;
use web3::contract::{self, Options};
use web3::ethabi::{self, Function, Param, ParamType, StateMutability};
use web3::types::Address;
use web3::Transport;

lazy_static! {
    static ref FACTORY_V2: ethabi::Contract = ethabi::Contract {
        constructor: None,
        functions: BTreeMap::from([(
            "getPair".to_string(),
            vec![Function {
                name: "getPair".to_string(),
                state_mutability: StateMutability::View,
                inputs: vec![
                    Param {
                        name: "".to_string(),
                        kind: ParamType::Address,
                        internal_type: Some("address".to_string()),
                    },
                    Param {
                        name: "".to_string(),
                        kind: ParamType::Address,
                        internal_type: Some("address".to_string()),
                    }
                ],
                outputs: vec![Param {
                    name: "".to_string(),
                    kind: ParamType::Address,
                    internal_type: Some("address".to_string()),
                }],
                constant: true,
            }],
        )]),
        events: BTreeMap::new(),
        errors: BTreeMap::new(),
        receive: false,
        fallback: false,
    };
}

#[derive(Clone)]
pub struct Factory<T: Transport> {
    contract: contract::Contract<T>,
}

impl<T: Transport> Factory<T> {
    pub fn new(eth: Eth<T>, address: Address) -> Self {
        Self {
            contract: contract::Contract::new(eth, address, FACTORY_V2.clone()),
        }
    }

    pub fn address(&self) -> Address {
        self.contract.address()
    }

    pub async fn get_pair(&self, (t0, t1): (Address, Address)) -> web3::contract::Result<Address> {
        self.contract
            .query("getPair", (t0, t1), None, Options::default(), None)
            .map_ok(|(a,)| a)
            .await
    }
}
