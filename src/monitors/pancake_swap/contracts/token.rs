use std::collections::BTreeMap;

use lazy_static::lazy_static;
use web3::api::Eth;
use web3::contract::{self, Options};
use web3::ethabi::{self, Function, Param, ParamType, StateMutability};
use web3::types::Address;
use web3::Transport;

lazy_static! {
    static ref TOKEN: ethabi::Contract = ethabi::Contract {
        constructor: None,
        functions: BTreeMap::from([
            (
                "decimals".to_string(),
                vec![
                    #[allow(deprecated)]
                    Function {
                        name: "decimals".to_string(),
                        state_mutability: StateMutability::View,
                        inputs: vec![],
                        outputs: vec![Param {
                            name: "".to_string(),
                            kind: ParamType::Uint(8),
                            internal_type: Some("uint8".to_string()),
                        }],
                        constant: true,
                    }
                ],
            ),
            (
                "name".to_string(),
                vec![
                    #[allow(deprecated)]
                    Function {
                        name: "name".to_string(),
                        state_mutability: StateMutability::View,
                        inputs: vec![],
                        outputs: vec![Param {
                            name: "".to_string(),
                            kind: ParamType::String,
                            internal_type: None,
                        }],
                        constant: true,
                    }
                ],
            )
        ]),
        events: BTreeMap::new(),
        errors: BTreeMap::new(),
        receive: false,
        fallback: false,
    };
}

#[derive(Clone)]
pub struct Token<T: Transport> {
    contract: contract::Contract<T>,
}

impl<T: Transport> Token<T> {
    pub fn new(eth: Eth<T>, address: Address) -> Self {
        Self {
            contract: contract::Contract::new(eth, address, TOKEN.clone()),
        }
    }

    pub fn address(&self) -> Address {
        self.contract.address()
    }

    pub async fn decimals(&self) -> web3::contract::Result<u8> {
        self.contract
            .query("decimals", (), None, Options::default(), None)
            .await
            .map(|(d,)| d)
    }

    pub async fn name(&self) -> web3::contract::Result<String> {
        self.contract
            .query("name", (), None, Options::default(), None)
            .await
            .map(|(s,)| s)
    }
}
