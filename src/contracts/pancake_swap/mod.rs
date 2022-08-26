use web3::ethabi::Address;
use web3::types::U256;

address!(pub ROUTER_V2_ADDRESS, "10ED43C718714eb63d5aA57B78B54704E256024E");
load_contract!(pub ROUTER_V2, "./router_v2.json");
contract_function!(
    pub ROUTER_V2_SWAP_EXACT_ETH_FOR_TOKENS,
    ROUTER_V2,
    "swapExactETHForTokens"
);

address!(pub FACTORY_V2_ADDRESS, "cA143Ce32Fe78f1f7019d7d551a6402fC5350c73");
load_contract!(pub FACTORY_V2, "./factory_v2.json");

load_contract!(pub PAIR, "./pair.json");
load_contract!(pub TOKEN, "./token.json");
