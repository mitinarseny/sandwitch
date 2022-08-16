use std::str::FromStr;

use lazy_static::lazy_static;
use web3::ethabi::{Address, Function, Param, ParamType, Token};
use web3::types::U256;

#[derive(Debug)]
pub struct SwapExactETHForTokensInputs {
    pub amount_out_min: U256,
    pub path: Vec<Address>,
    pub to: Address,
    pub deadline: U256,
}

impl TryFrom<&[u8]> for SwapExactETHForTokensInputs {
    type Error = web3::ethabi::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        SWAP_EXACT_ETH_FOR_TOKENS.decode_input(value).map(|v| {
            let mut i = v.into_iter();
            Self {
                amount_out_min: i.next().unwrap().into_uint().unwrap(),
                path: i
                    .next()
                    .unwrap()
                    .into_array()
                    .unwrap()
                    .into_iter()
                    .map(Token::into_address)
                    .try_collect::<Vec<_>>()
                    .unwrap(),
                to: i.next().unwrap().into_address().unwrap(),
                deadline: i.next().unwrap().into_uint().unwrap(),
            }
        })
    }
}

lazy_static! {
    pub static ref ADDRESS: Address =
        Address::from_str("0x10ED43C718714eb63d5aA57B78B54704E256024E").unwrap();

    pub static ref SWAP_EXACT_ETH_FOR_TOKENS: Function = Function {
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
