use std::collections::BTreeMap;

use lazy_static::lazy_static;
use web3::ethabi::{Contract, Function, Param, ParamType, StateMutability};

lazy_static! {
    pub static ref SWAP_EXACT_ETH_FOR_TOKENS: Function = Function {
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
    pub static ref ROUTER_V2: Contract = Contract {
        constructor: None,
        functions: BTreeMap::from([(
            SWAP_EXACT_ETH_FOR_TOKENS.name.clone(),
            vec![SWAP_EXACT_ETH_FOR_TOKENS.clone()]
        )]),
        events: BTreeMap::new(),
        errors: BTreeMap::new(),
        receive: false,
        fallback: false,
    };
}

address!(pub ADDRESS, "10ED43C718714eb63d5aA57B78B54704E256024E");
