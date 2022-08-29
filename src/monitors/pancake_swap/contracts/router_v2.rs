use std::collections::BTreeMap;

use lazy_static::lazy_static;
use web3::api::Eth;
use web3::contract::{self, Options};
use web3::ethabi::{self, Function, Param, ParamType, StateMutability};
use web3::types::Address;
use web3::Transport;

lazy_static! {
    pub static ref SWAP_EXACT_ETH_FOR_TOKENS: Function = #[allow(deprecated)]
    Function {
        name: "swapExactETHForTokens".to_string(),
        state_mutability: StateMutability::Payable,
        inputs: vec![
            Param {
                name: "amountOutMin".to_string(),
                kind: ParamType::Uint(256),
                internal_type: Some("uint256".to_string()),
            },
            Param {
                name: "path".to_string(),
                kind: ParamType::Array(Box::new(ParamType::Address)),
                internal_type: Some("address[]".to_string()),
            },
            Param {
                name: "to".to_string(),
                kind: ParamType::Address,
                internal_type: Some("address".to_string()),
            },
            Param {
                name: "deadline".to_string(),
                kind: ParamType::Uint(256),
                internal_type: Some("uint256".to_string()),
            },
        ],
        outputs: vec![Param {
            name: "amounts".to_string(),
            kind: ParamType::Array(Box::new(ParamType::Uint(256))),
            internal_type: Some("uint256[]".to_string()),
        }],
        constant: false,
    };
    static ref ROUTER_V2: ethabi::Contract = ethabi::Contract {
        constructor: None,
        functions: BTreeMap::from([
            (
                SWAP_EXACT_ETH_FOR_TOKENS.name.clone(),
                vec![SWAP_EXACT_ETH_FOR_TOKENS.clone()]
            ),
            (
                "factory".to_string(),
                vec![
                    #[allow(deprecated)]
                    Function {
                        name: "factory".to_string(),
                        state_mutability: StateMutability::Pure,
                        inputs: vec![],
                        outputs: vec![Param {
                            name: "".to_string(),
                            kind: ParamType::Address,
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
pub struct Router<T: Transport> {
    contract: contract::Contract<T>,
}

impl<T: Transport> Router<T> {
    pub fn new(eth: Eth<T>, address: Address) -> Self {
        Self {
            contract: contract::Contract::new(eth, address, ROUTER_V2.clone()),
        }
    }

    pub fn address(&self) -> Address {
        self.contract.address()
    }

    pub async fn factory(&self) -> web3::contract::Result<Address> {
        self.contract
            .query("factory", (), None, Options::default(), None)
            .await
            .map(|(a,)| a)
    }
}
