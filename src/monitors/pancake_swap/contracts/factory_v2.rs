use std::collections::BTreeMap;

use lazy_static::lazy_static;
use web3::ethabi::{Contract, Function, Param, ParamType, StateMutability};

lazy_static! {
    pub static ref FACTORY_V2: Contract = Contract {
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

address!(pub ADDRESS, "cA143Ce32Fe78f1f7019d7d551a6402fC5350c73");
