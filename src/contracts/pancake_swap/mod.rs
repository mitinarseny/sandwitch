use web3::ethabi::{Address, Token};
use web3::types::U256;

address!(pub ROUTER_V2_ADDRESS, "10ED43C718714eb63d5aA57B78B54704E256024E");
load_contract!(pub ROUTER_V2, "./router_v2.json");
contract_function!(
    pub ROUTER_V2_SWAP_EXACT_ETH_FOR_TOKENS,
    ROUTER_V2,
    "swapExactETHForTokens"
);

#[derive(Debug)]
pub struct RouterV2SwapExactETHForTokensInputs {
    pub amount_out_min: U256,
    pub path: Vec<Address>,
    pub to: Address,
    pub deadline: U256,
}

impl TryFrom<&[u8]> for RouterV2SwapExactETHForTokensInputs {
    type Error = web3::ethabi::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        ROUTER_V2_SWAP_EXACT_ETH_FOR_TOKENS
            .decode_input(value)
            .map(|v| {
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

address!(pub FACTORY_V2_ADDRESS, "cA143Ce32Fe78f1f7019d7d551a6402fC5350c73");
load_contract!(pub FACTORY_V2, "./factory_v2.json");

load_contract!(pub PAIR, "./pair.json");
