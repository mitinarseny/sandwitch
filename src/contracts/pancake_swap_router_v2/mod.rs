use futures::future::{BoxFuture};
use futures::future;
use lazy_static::lazy_static;
use std::str::FromStr;
use web3::api::Eth;
use web3::ethabi::{Function, Param, ParamType};

use web3::types::Address;
use web3::Transport;

use super::SwapContract;

lazy_static! {
    static ref SWAP_EXACT_ETH_FOR_TOKENS: Function = Function {
        name: "swapExactETHForTokens".to_string(),
        inputs: vec![
            Param {
                name: "amountOutMin".to_string(),
                kind: ParamType::Uint(256),
                internal_type: "uint256".to_string().into(),
            },
            Param {
                name: "path".to_string(),
                kind: ParamType::Array(Box::new(ParamType::Address)),
                internal_type: "address[]".to_string().into(),
            },
            Param {
                name: "to".to_string(),
                kind: ParamType::Address,
                internal_type: "address".to_string().into(),
            },
            Param {
                name: "deadline".to_string(),
                kind: ParamType::Uint(256),
                internal_type: "uint256".to_string().into(),
            },
        ],
        outputs: vec![Param {
            name: "amounts".to_string(),
            kind: ParamType::Array(Box::new(ParamType::Uint(256))),
            internal_type: "uint256[]".to_string().into(),
        }],
        state_mutability: web3::ethabi::StateMutability::Payable,
        constant: false,
    };
}

pub struct PancakeSwapRouterV2 {}

impl SwapContract for PancakeSwapRouterV2 {
    fn process(&mut self, input: &[u8]) -> BoxFuture<'_, ()> {

        if !input.starts_with(&SWAP_EXACT_ETH_FOR_TOKENS.short_signature()) {
            return Box::pin(future::ready(()));
        }
        let tokens = SWAP_EXACT_ETH_FOR_TOKENS.decode_input(&input[4..]).unwrap();
        println!("{:?}", tokens[1]);
        Box::pin(future::ready(()))
    }
}
