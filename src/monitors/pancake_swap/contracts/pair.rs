use std::collections::BTreeMap;

use futures::TryFutureExt;
use lazy_static::lazy_static;
use web3::api::Eth;
use web3::contract::{self, Options};
use web3::ethabi::{self, Function, Param, ParamType, StateMutability};
use web3::types::Address;
use web3::Transport;

lazy_static! {
    static ref PAIR: ethabi::Contract = ethabi::Contract {
        constructor: None,
        functions: BTreeMap::from([(
            "getReserves".to_string(),
            vec![Function {
                name: "getReserves".to_string(),
                state_mutability: StateMutability::View,
                inputs: vec![],
                outputs: vec![
                    Param {
                        name: "_reserve0".to_string(),
                        kind: ParamType::Uint(112),
                        internal_type: Some("uint112".to_string()),
                    },
                    Param {
                        name: "_reserve1".to_string(),
                        kind: ParamType::Uint(112),
                        internal_type: Some("uint112".to_string()),
                    },
                    Param {
                        name: "_blockTimestampLast".to_string(),
                        kind: ParamType::Uint(32),
                        internal_type: Some("uint32".to_string()),
                    },
                ],
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
pub struct Pair<T: Transport> {
    contract: contract::Contract<T>,
}

impl<T: Transport> Pair<T> {
    pub fn new(eth: Eth<T>, address: Address) -> Self {
        Self {
            contract: contract::Contract::new(eth, address, PAIR.clone()),
        }
    }

    pub fn address(&self) -> Address {
        self.contract.address()
    }

    pub async fn get_reserves(&self) -> web3::contract::Result<(u128, u128, u32)> {
        self.contract
            .query("getReserves", (), None, Options::default(), None)
            .await
    }
}
