use std::str::FromStr;

use lazy_static::lazy_static;
use web3::ethabi::{Address, Function, Param, ParamType, Token};
use web3::types::U256;



lazy_static! {
    pub static ref ROUTER_V2: web3::ethabi::Contract = web3::ethabi::Contract::load(
        include_bytes!(
            "./contracts/pancake_swap_router_v2/0x10ED43C718714eb63d5aA57B78B54704E256024E.json"
        )
        .as_slice()
    )
    .unwrap();
    
    pub static ref FACTORY_V2: web3::ethabi::Contract = web3::ethabi::Contract::load(
        include_bytes!("./contracts/0xcA143Ce32Fe78f1f7019d7d551a6402fC5350c73.json").as_slice()
    )
    .unwrap();

    // pub static ref 
    // pub static ref ADDRESS: Address =
    //     Address::from_str("0x10ED43C718714eb63d5aA57B78B54704E256024E").unwrap();
    // pub static ref SWAP_EXACT_ETH_FOR_TOKENS: Function = Function {
    //     name: "swapExactETHForTokens".to_string(),
    //     inputs: vec![
    //         Param {
    //             name: "amountOutMin".to_string(),
    //             kind: ParamType::Uint(256),
    //             internal_type: "uint256".to_string().into(),
    //         },
    //         Param {
    //             name: "path".to_string(),
    //             kind: ParamType::Array(Box::new(ParamType::Address)),
    //             internal_type: "address[]".to_string().into(),
    //         },
    //         Param {
    //             name: "to".to_string(),
    //             kind: ParamType::Address,
    //             internal_type: "address".to_string().into(),
    //         },
    //         Param {
    //             name: "deadline".to_string(),
    //             kind: ParamType::Uint(256),
    //             internal_type: "uint256".to_string().into(),
    //         },
    //     ],
    //     outputs: vec![Param {
    //         name: "amounts".to_string(),
    //         kind: ParamType::Array(Box::new(ParamType::Uint(256))),
    //         internal_type: "uint256[]".to_string().into(),
    //     }],
    //     state_mutability: web3::ethabi::StateMutability::Payable,
    //     constant: false,
    // };
    // pub static ref GET_RESERVES: Function = Function {
    //     name: "getReserves".to_string(),
    //     inputs: Vec::new(),
    //     outputs: vec![
    //         Param {
    //             name: "reserve0".to_string(),
    //             kind: ParamType::Uint(112),
    //             internal_type: "uint112".to_string().into(),
    //         },
    //         Param {
    //             name: "reserve1".to_string(),
    //             kind: ParamType::Uint(112),
    //             internal_type: "uint112".to_string().into(),
    //         },
    //         Param {
    //             name: "blockTimestampLast".to_string(),
    //             kind: ParamType::Uint(32),
    //             internal_type: "uint32".to_string().into(),
    //         },
    //     ],
    //     state_mutability: web3::ethabi::StateMutability::View,
    //     constant: false,
    // };
}
