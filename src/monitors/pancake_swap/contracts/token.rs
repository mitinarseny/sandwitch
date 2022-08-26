use std::collections::BTreeMap;

use lazy_static::lazy_static;
use web3::ethabi::{Contract, Function, Param, ParamType, StateMutability};

lazy_static! {
    pub static ref TOKEN: Contract = Contract {
        constructor: None,
        functions: BTreeMap::from([(
            "decimals".to_string(),
            vec![Function {
                name: "decimals".to_string(),
                state_mutability: StateMutability::View,
                inputs: vec![],
                outputs: vec![Param {
                    name: "".to_string(),
                    kind: ParamType::Uint(8),
                    internal_type: Some("uint8".to_string()),
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
