use std::collections::BTreeMap;

use lazy_static::lazy_static;
use web3::ethabi::{Contract, Function, Param, ParamType, StateMutability};

lazy_static! {
    pub static ref PAIR: Contract = Contract {
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
