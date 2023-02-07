pub use i_pancake_router_01::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod i_pancake_router_01 {
    #![allow(clippy::enum_variant_names)]
    #![allow(dead_code)]
    #![allow(clippy::type_complexity)]
    #![allow(unused_imports)]
    use ethers::contract::{
        builders::{ContractCall, Event},
        Contract, Lazy,
    };
    use ethers::core::{
        abi::{Abi, Detokenize, InvalidOutputType, Token, Tokenizable},
        types::*,
    };
    use ethers::providers::Middleware;
    #[doc = "IPancakeRouter01 was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[],\"stateMutability\":\"pure\",\"type\":\"function\",\"name\":\"WETH\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"tokenA\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"tokenB\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountADesired\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountBDesired\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountAMin\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountBMin\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"deadline\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"addLiquidity\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"amountA\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountB\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"liquidity\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"token\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountTokenDesired\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountTokenMin\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountETHMin\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"deadline\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"payable\",\"type\":\"function\",\"name\":\"addLiquidityETH\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"amountToken\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountETH\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"liquidity\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"pure\",\"type\":\"function\",\"name\":\"factory\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"amountOut\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"reserveIn\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"reserveOut\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"pure\",\"type\":\"function\",\"name\":\"getAmountIn\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"amountIn\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"amountIn\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"reserveIn\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"reserveOut\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"pure\",\"type\":\"function\",\"name\":\"getAmountOut\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"amountOut\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"amountOut\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"path\",\"type\":\"address[]\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getAmountsIn\",\"outputs\":[{\"internalType\":\"uint256[]\",\"name\":\"amounts\",\"type\":\"uint256[]\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"amountIn\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"path\",\"type\":\"address[]\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getAmountsOut\",\"outputs\":[{\"internalType\":\"uint256[]\",\"name\":\"amounts\",\"type\":\"uint256[]\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"amountA\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"reserveA\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"reserveB\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"pure\",\"type\":\"function\",\"name\":\"quote\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"amountB\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"tokenA\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"tokenB\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"liquidity\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountAMin\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountBMin\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"deadline\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"removeLiquidity\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"amountA\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountB\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"token\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"liquidity\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountTokenMin\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountETHMin\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"deadline\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"removeLiquidityETH\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"amountToken\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountETH\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"token\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"liquidity\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountTokenMin\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountETHMin\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"deadline\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bool\",\"name\":\"approveMax\",\"type\":\"bool\",\"components\":[]},{\"internalType\":\"uint8\",\"name\":\"v\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"r\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"s\",\"type\":\"bytes32\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"removeLiquidityETHWithPermit\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"amountToken\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountETH\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"tokenA\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"tokenB\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"liquidity\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountAMin\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountBMin\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"deadline\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bool\",\"name\":\"approveMax\",\"type\":\"bool\",\"components\":[]},{\"internalType\":\"uint8\",\"name\":\"v\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"r\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"s\",\"type\":\"bytes32\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"removeLiquidityWithPermit\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"amountA\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountB\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"amountOut\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"path\",\"type\":\"address[]\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"deadline\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"payable\",\"type\":\"function\",\"name\":\"swapETHForExactTokens\",\"outputs\":[{\"internalType\":\"uint256[]\",\"name\":\"amounts\",\"type\":\"uint256[]\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"amountOutMin\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"path\",\"type\":\"address[]\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"deadline\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"payable\",\"type\":\"function\",\"name\":\"swapExactETHForTokens\",\"outputs\":[{\"internalType\":\"uint256[]\",\"name\":\"amounts\",\"type\":\"uint256[]\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"amountIn\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountOutMin\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"path\",\"type\":\"address[]\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"deadline\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"swapExactTokensForETH\",\"outputs\":[{\"internalType\":\"uint256[]\",\"name\":\"amounts\",\"type\":\"uint256[]\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"amountIn\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountOutMin\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"path\",\"type\":\"address[]\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"deadline\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"swapExactTokensForTokens\",\"outputs\":[{\"internalType\":\"uint256[]\",\"name\":\"amounts\",\"type\":\"uint256[]\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"amountOut\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountInMax\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"path\",\"type\":\"address[]\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"deadline\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"swapTokensForExactETH\",\"outputs\":[{\"internalType\":\"uint256[]\",\"name\":\"amounts\",\"type\":\"uint256[]\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"amountOut\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountInMax\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"path\",\"type\":\"address[]\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"deadline\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"swapTokensForExactTokens\",\"outputs\":[{\"internalType\":\"uint256[]\",\"name\":\"amounts\",\"type\":\"uint256[]\",\"components\":[]}]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static IPANCAKEROUTER01_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    pub struct IPancakeRouter01<M>(ethers::contract::Contract<M>);
    impl<M> Clone for IPancakeRouter01<M> {
        fn clone(&self) -> Self {
            IPancakeRouter01(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for IPancakeRouter01<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for IPancakeRouter01<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(IPancakeRouter01))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> IPancakeRouter01<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), IPANCAKEROUTER01_ABI.clone(), client)
                .into()
        }
        #[doc = "Calls the contract's `WETH` (0xad5c4648) function"]
        pub fn weth(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([173, 92, 70, 72], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `addLiquidity` (0xe8e33700) function"]
        pub fn add_liquidity(
            &self,
            token_a: ethers::core::types::Address,
            token_b: ethers::core::types::Address,
            amount_a_desired: ethers::core::types::U256,
            amount_b_desired: ethers::core::types::U256,
            amount_a_min: ethers::core::types::U256,
            amount_b_min: ethers::core::types::U256,
            to: ethers::core::types::Address,
            deadline: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<
            M,
            (
                ethers::core::types::U256,
                ethers::core::types::U256,
                ethers::core::types::U256,
            ),
        > {
            self.0
                .method_hash(
                    [232, 227, 55, 0],
                    (
                        token_a,
                        token_b,
                        amount_a_desired,
                        amount_b_desired,
                        amount_a_min,
                        amount_b_min,
                        to,
                        deadline,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `addLiquidityETH` (0xf305d719) function"]
        pub fn add_liquidity_eth(
            &self,
            token: ethers::core::types::Address,
            amount_token_desired: ethers::core::types::U256,
            amount_token_min: ethers::core::types::U256,
            amount_eth_min: ethers::core::types::U256,
            to: ethers::core::types::Address,
            deadline: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<
            M,
            (
                ethers::core::types::U256,
                ethers::core::types::U256,
                ethers::core::types::U256,
            ),
        > {
            self.0
                .method_hash(
                    [243, 5, 215, 25],
                    (
                        token,
                        amount_token_desired,
                        amount_token_min,
                        amount_eth_min,
                        to,
                        deadline,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `factory` (0xc45a0155) function"]
        pub fn factory(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([196, 90, 1, 85], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getAmountIn` (0x85f8c259) function"]
        pub fn get_amount_in(
            &self,
            amount_out: ethers::core::types::U256,
            reserve_in: ethers::core::types::U256,
            reserve_out: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([133, 248, 194, 89], (amount_out, reserve_in, reserve_out))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getAmountOut` (0x054d50d4) function"]
        pub fn get_amount_out(
            &self,
            amount_in: ethers::core::types::U256,
            reserve_in: ethers::core::types::U256,
            reserve_out: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([5, 77, 80, 212], (amount_in, reserve_in, reserve_out))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getAmountsIn` (0x1f00ca74) function"]
        pub fn get_amounts_in(
            &self,
            amount_out: ethers::core::types::U256,
            path: ::std::vec::Vec<ethers::core::types::Address>,
        ) -> ethers::contract::builders::ContractCall<M, ::std::vec::Vec<ethers::core::types::U256>>
        {
            self.0
                .method_hash([31, 0, 202, 116], (amount_out, path))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getAmountsOut` (0xd06ca61f) function"]
        pub fn get_amounts_out(
            &self,
            amount_in: ethers::core::types::U256,
            path: ::std::vec::Vec<ethers::core::types::Address>,
        ) -> ethers::contract::builders::ContractCall<M, ::std::vec::Vec<ethers::core::types::U256>>
        {
            self.0
                .method_hash([208, 108, 166, 31], (amount_in, path))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `quote` (0xad615dec) function"]
        pub fn quote(
            &self,
            amount_a: ethers::core::types::U256,
            reserve_a: ethers::core::types::U256,
            reserve_b: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([173, 97, 93, 236], (amount_a, reserve_a, reserve_b))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `removeLiquidity` (0xbaa2abde) function"]
        pub fn remove_liquidity(
            &self,
            token_a: ethers::core::types::Address,
            token_b: ethers::core::types::Address,
            liquidity: ethers::core::types::U256,
            amount_a_min: ethers::core::types::U256,
            amount_b_min: ethers::core::types::U256,
            to: ethers::core::types::Address,
            deadline: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<
            M,
            (ethers::core::types::U256, ethers::core::types::U256),
        > {
            self.0
                .method_hash(
                    [186, 162, 171, 222],
                    (
                        token_a,
                        token_b,
                        liquidity,
                        amount_a_min,
                        amount_b_min,
                        to,
                        deadline,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `removeLiquidityETH` (0x02751cec) function"]
        pub fn remove_liquidity_eth(
            &self,
            token: ethers::core::types::Address,
            liquidity: ethers::core::types::U256,
            amount_token_min: ethers::core::types::U256,
            amount_eth_min: ethers::core::types::U256,
            to: ethers::core::types::Address,
            deadline: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<
            M,
            (ethers::core::types::U256, ethers::core::types::U256),
        > {
            self.0
                .method_hash(
                    [2, 117, 28, 236],
                    (
                        token,
                        liquidity,
                        amount_token_min,
                        amount_eth_min,
                        to,
                        deadline,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `removeLiquidityETHWithPermit` (0xded9382a) function"]
        pub fn remove_liquidity_eth_with_permit(
            &self,
            token: ethers::core::types::Address,
            liquidity: ethers::core::types::U256,
            amount_token_min: ethers::core::types::U256,
            amount_eth_min: ethers::core::types::U256,
            to: ethers::core::types::Address,
            deadline: ethers::core::types::U256,
            approve_max: bool,
            v: u8,
            r: [u8; 32],
            s: [u8; 32],
        ) -> ethers::contract::builders::ContractCall<
            M,
            (ethers::core::types::U256, ethers::core::types::U256),
        > {
            self.0
                .method_hash(
                    [222, 217, 56, 42],
                    (
                        token,
                        liquidity,
                        amount_token_min,
                        amount_eth_min,
                        to,
                        deadline,
                        approve_max,
                        v,
                        r,
                        s,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `removeLiquidityWithPermit` (0x2195995c) function"]
        pub fn remove_liquidity_with_permit(
            &self,
            token_a: ethers::core::types::Address,
            token_b: ethers::core::types::Address,
            liquidity: ethers::core::types::U256,
            amount_a_min: ethers::core::types::U256,
            amount_b_min: ethers::core::types::U256,
            to: ethers::core::types::Address,
            deadline: ethers::core::types::U256,
            approve_max: bool,
            v: u8,
            r: [u8; 32],
            s: [u8; 32],
        ) -> ethers::contract::builders::ContractCall<
            M,
            (ethers::core::types::U256, ethers::core::types::U256),
        > {
            self.0
                .method_hash(
                    [33, 149, 153, 92],
                    (
                        token_a,
                        token_b,
                        liquidity,
                        amount_a_min,
                        amount_b_min,
                        to,
                        deadline,
                        approve_max,
                        v,
                        r,
                        s,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `swapETHForExactTokens` (0xfb3bdb41) function"]
        pub fn swap_eth_for_exact_tokens(
            &self,
            amount_out: ethers::core::types::U256,
            path: ::std::vec::Vec<ethers::core::types::Address>,
            to: ethers::core::types::Address,
            deadline: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ::std::vec::Vec<ethers::core::types::U256>>
        {
            self.0
                .method_hash([251, 59, 219, 65], (amount_out, path, to, deadline))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `swapExactETHForTokens` (0x7ff36ab5) function"]
        pub fn swap_exact_eth_for_tokens(
            &self,
            amount_out_min: ethers::core::types::U256,
            path: ::std::vec::Vec<ethers::core::types::Address>,
            to: ethers::core::types::Address,
            deadline: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ::std::vec::Vec<ethers::core::types::U256>>
        {
            self.0
                .method_hash([127, 243, 106, 181], (amount_out_min, path, to, deadline))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `swapExactTokensForETH` (0x18cbafe5) function"]
        pub fn swap_exact_tokens_for_eth(
            &self,
            amount_in: ethers::core::types::U256,
            amount_out_min: ethers::core::types::U256,
            path: ::std::vec::Vec<ethers::core::types::Address>,
            to: ethers::core::types::Address,
            deadline: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ::std::vec::Vec<ethers::core::types::U256>>
        {
            self.0
                .method_hash(
                    [24, 203, 175, 229],
                    (amount_in, amount_out_min, path, to, deadline),
                )
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `swapExactTokensForTokens` (0x38ed1739) function"]
        pub fn swap_exact_tokens_for_tokens(
            &self,
            amount_in: ethers::core::types::U256,
            amount_out_min: ethers::core::types::U256,
            path: ::std::vec::Vec<ethers::core::types::Address>,
            to: ethers::core::types::Address,
            deadline: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ::std::vec::Vec<ethers::core::types::U256>>
        {
            self.0
                .method_hash(
                    [56, 237, 23, 57],
                    (amount_in, amount_out_min, path, to, deadline),
                )
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `swapTokensForExactETH` (0x4a25d94a) function"]
        pub fn swap_tokens_for_exact_eth(
            &self,
            amount_out: ethers::core::types::U256,
            amount_in_max: ethers::core::types::U256,
            path: ::std::vec::Vec<ethers::core::types::Address>,
            to: ethers::core::types::Address,
            deadline: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ::std::vec::Vec<ethers::core::types::U256>>
        {
            self.0
                .method_hash(
                    [74, 37, 217, 74],
                    (amount_out, amount_in_max, path, to, deadline),
                )
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `swapTokensForExactTokens` (0x8803dbee) function"]
        pub fn swap_tokens_for_exact_tokens(
            &self,
            amount_out: ethers::core::types::U256,
            amount_in_max: ethers::core::types::U256,
            path: ::std::vec::Vec<ethers::core::types::Address>,
            to: ethers::core::types::Address,
            deadline: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ::std::vec::Vec<ethers::core::types::U256>>
        {
            self.0
                .method_hash(
                    [136, 3, 219, 238],
                    (amount_out, amount_in_max, path, to, deadline),
                )
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for IPancakeRouter01<M> {
        fn from(contract: ethers::contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
    #[doc = "Container type for all input parameters for the `WETH` function with signature `WETH()` and selector `[173, 92, 70, 72]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "WETH", abi = "WETH()")]
    pub struct WethCall;
    #[doc = "Container type for all input parameters for the `addLiquidity` function with signature `addLiquidity(address,address,uint256,uint256,uint256,uint256,address,uint256)` and selector `[232, 227, 55, 0]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(
        name = "addLiquidity",
        abi = "addLiquidity(address,address,uint256,uint256,uint256,uint256,address,uint256)"
    )]
    pub struct AddLiquidityCall {
        pub token_a: ethers::core::types::Address,
        pub token_b: ethers::core::types::Address,
        pub amount_a_desired: ethers::core::types::U256,
        pub amount_b_desired: ethers::core::types::U256,
        pub amount_a_min: ethers::core::types::U256,
        pub amount_b_min: ethers::core::types::U256,
        pub to: ethers::core::types::Address,
        pub deadline: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `addLiquidityETH` function with signature `addLiquidityETH(address,uint256,uint256,uint256,address,uint256)` and selector `[243, 5, 215, 25]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(
        name = "addLiquidityETH",
        abi = "addLiquidityETH(address,uint256,uint256,uint256,address,uint256)"
    )]
    pub struct AddLiquidityETHCall {
        pub token: ethers::core::types::Address,
        pub amount_token_desired: ethers::core::types::U256,
        pub amount_token_min: ethers::core::types::U256,
        pub amount_eth_min: ethers::core::types::U256,
        pub to: ethers::core::types::Address,
        pub deadline: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `factory` function with signature `factory()` and selector `[196, 90, 1, 85]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "factory", abi = "factory()")]
    pub struct FactoryCall;
    #[doc = "Container type for all input parameters for the `getAmountIn` function with signature `getAmountIn(uint256,uint256,uint256)` and selector `[133, 248, 194, 89]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "getAmountIn", abi = "getAmountIn(uint256,uint256,uint256)")]
    pub struct GetAmountInCall {
        pub amount_out: ethers::core::types::U256,
        pub reserve_in: ethers::core::types::U256,
        pub reserve_out: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `getAmountOut` function with signature `getAmountOut(uint256,uint256,uint256)` and selector `[5, 77, 80, 212]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "getAmountOut", abi = "getAmountOut(uint256,uint256,uint256)")]
    pub struct GetAmountOutCall {
        pub amount_in: ethers::core::types::U256,
        pub reserve_in: ethers::core::types::U256,
        pub reserve_out: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `getAmountsIn` function with signature `getAmountsIn(uint256,address[])` and selector `[31, 0, 202, 116]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "getAmountsIn", abi = "getAmountsIn(uint256,address[])")]
    pub struct GetAmountsInCall {
        pub amount_out: ethers::core::types::U256,
        pub path: ::std::vec::Vec<ethers::core::types::Address>,
    }
    #[doc = "Container type for all input parameters for the `getAmountsOut` function with signature `getAmountsOut(uint256,address[])` and selector `[208, 108, 166, 31]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "getAmountsOut", abi = "getAmountsOut(uint256,address[])")]
    pub struct GetAmountsOutCall {
        pub amount_in: ethers::core::types::U256,
        pub path: ::std::vec::Vec<ethers::core::types::Address>,
    }
    #[doc = "Container type for all input parameters for the `quote` function with signature `quote(uint256,uint256,uint256)` and selector `[173, 97, 93, 236]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "quote", abi = "quote(uint256,uint256,uint256)")]
    pub struct QuoteCall {
        pub amount_a: ethers::core::types::U256,
        pub reserve_a: ethers::core::types::U256,
        pub reserve_b: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `removeLiquidity` function with signature `removeLiquidity(address,address,uint256,uint256,uint256,address,uint256)` and selector `[186, 162, 171, 222]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(
        name = "removeLiquidity",
        abi = "removeLiquidity(address,address,uint256,uint256,uint256,address,uint256)"
    )]
    pub struct RemoveLiquidityCall {
        pub token_a: ethers::core::types::Address,
        pub token_b: ethers::core::types::Address,
        pub liquidity: ethers::core::types::U256,
        pub amount_a_min: ethers::core::types::U256,
        pub amount_b_min: ethers::core::types::U256,
        pub to: ethers::core::types::Address,
        pub deadline: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `removeLiquidityETH` function with signature `removeLiquidityETH(address,uint256,uint256,uint256,address,uint256)` and selector `[2, 117, 28, 236]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(
        name = "removeLiquidityETH",
        abi = "removeLiquidityETH(address,uint256,uint256,uint256,address,uint256)"
    )]
    pub struct RemoveLiquidityETHCall {
        pub token: ethers::core::types::Address,
        pub liquidity: ethers::core::types::U256,
        pub amount_token_min: ethers::core::types::U256,
        pub amount_eth_min: ethers::core::types::U256,
        pub to: ethers::core::types::Address,
        pub deadline: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `removeLiquidityETHWithPermit` function with signature `removeLiquidityETHWithPermit(address,uint256,uint256,uint256,address,uint256,bool,uint8,bytes32,bytes32)` and selector `[222, 217, 56, 42]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(
        name = "removeLiquidityETHWithPermit",
        abi = "removeLiquidityETHWithPermit(address,uint256,uint256,uint256,address,uint256,bool,uint8,bytes32,bytes32)"
    )]
    pub struct RemoveLiquidityETHWithPermitCall {
        pub token: ethers::core::types::Address,
        pub liquidity: ethers::core::types::U256,
        pub amount_token_min: ethers::core::types::U256,
        pub amount_eth_min: ethers::core::types::U256,
        pub to: ethers::core::types::Address,
        pub deadline: ethers::core::types::U256,
        pub approve_max: bool,
        pub v: u8,
        pub r: [u8; 32],
        pub s: [u8; 32],
    }
    #[doc = "Container type for all input parameters for the `removeLiquidityWithPermit` function with signature `removeLiquidityWithPermit(address,address,uint256,uint256,uint256,address,uint256,bool,uint8,bytes32,bytes32)` and selector `[33, 149, 153, 92]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(
        name = "removeLiquidityWithPermit",
        abi = "removeLiquidityWithPermit(address,address,uint256,uint256,uint256,address,uint256,bool,uint8,bytes32,bytes32)"
    )]
    pub struct RemoveLiquidityWithPermitCall {
        pub token_a: ethers::core::types::Address,
        pub token_b: ethers::core::types::Address,
        pub liquidity: ethers::core::types::U256,
        pub amount_a_min: ethers::core::types::U256,
        pub amount_b_min: ethers::core::types::U256,
        pub to: ethers::core::types::Address,
        pub deadline: ethers::core::types::U256,
        pub approve_max: bool,
        pub v: u8,
        pub r: [u8; 32],
        pub s: [u8; 32],
    }
    #[doc = "Container type for all input parameters for the `swapETHForExactTokens` function with signature `swapETHForExactTokens(uint256,address[],address,uint256)` and selector `[251, 59, 219, 65]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(
        name = "swapETHForExactTokens",
        abi = "swapETHForExactTokens(uint256,address[],address,uint256)"
    )]
    pub struct SwapETHForExactTokensCall {
        pub amount_out: ethers::core::types::U256,
        pub path: ::std::vec::Vec<ethers::core::types::Address>,
        pub to: ethers::core::types::Address,
        pub deadline: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `swapExactETHForTokens` function with signature `swapExactETHForTokens(uint256,address[],address,uint256)` and selector `[127, 243, 106, 181]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(
        name = "swapExactETHForTokens",
        abi = "swapExactETHForTokens(uint256,address[],address,uint256)"
    )]
    pub struct SwapExactETHForTokensCall {
        pub amount_out_min: ethers::core::types::U256,
        pub path: ::std::vec::Vec<ethers::core::types::Address>,
        pub to: ethers::core::types::Address,
        pub deadline: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `swapExactTokensForETH` function with signature `swapExactTokensForETH(uint256,uint256,address[],address,uint256)` and selector `[24, 203, 175, 229]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(
        name = "swapExactTokensForETH",
        abi = "swapExactTokensForETH(uint256,uint256,address[],address,uint256)"
    )]
    pub struct SwapExactTokensForETHCall {
        pub amount_in: ethers::core::types::U256,
        pub amount_out_min: ethers::core::types::U256,
        pub path: ::std::vec::Vec<ethers::core::types::Address>,
        pub to: ethers::core::types::Address,
        pub deadline: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `swapExactTokensForTokens` function with signature `swapExactTokensForTokens(uint256,uint256,address[],address,uint256)` and selector `[56, 237, 23, 57]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(
        name = "swapExactTokensForTokens",
        abi = "swapExactTokensForTokens(uint256,uint256,address[],address,uint256)"
    )]
    pub struct SwapExactTokensForTokensCall {
        pub amount_in: ethers::core::types::U256,
        pub amount_out_min: ethers::core::types::U256,
        pub path: ::std::vec::Vec<ethers::core::types::Address>,
        pub to: ethers::core::types::Address,
        pub deadline: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `swapTokensForExactETH` function with signature `swapTokensForExactETH(uint256,uint256,address[],address,uint256)` and selector `[74, 37, 217, 74]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(
        name = "swapTokensForExactETH",
        abi = "swapTokensForExactETH(uint256,uint256,address[],address,uint256)"
    )]
    pub struct SwapTokensForExactETHCall {
        pub amount_out: ethers::core::types::U256,
        pub amount_in_max: ethers::core::types::U256,
        pub path: ::std::vec::Vec<ethers::core::types::Address>,
        pub to: ethers::core::types::Address,
        pub deadline: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `swapTokensForExactTokens` function with signature `swapTokensForExactTokens(uint256,uint256,address[],address,uint256)` and selector `[136, 3, 219, 238]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(
        name = "swapTokensForExactTokens",
        abi = "swapTokensForExactTokens(uint256,uint256,address[],address,uint256)"
    )]
    pub struct SwapTokensForExactTokensCall {
        pub amount_out: ethers::core::types::U256,
        pub amount_in_max: ethers::core::types::U256,
        pub path: ::std::vec::Vec<ethers::core::types::Address>,
        pub to: ethers::core::types::Address,
        pub deadline: ethers::core::types::U256,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum IPancakeRouter01Calls {
        Weth(WethCall),
        AddLiquidity(AddLiquidityCall),
        AddLiquidityETH(AddLiquidityETHCall),
        Factory(FactoryCall),
        GetAmountIn(GetAmountInCall),
        GetAmountOut(GetAmountOutCall),
        GetAmountsIn(GetAmountsInCall),
        GetAmountsOut(GetAmountsOutCall),
        Quote(QuoteCall),
        RemoveLiquidity(RemoveLiquidityCall),
        RemoveLiquidityETH(RemoveLiquidityETHCall),
        RemoveLiquidityETHWithPermit(RemoveLiquidityETHWithPermitCall),
        RemoveLiquidityWithPermit(RemoveLiquidityWithPermitCall),
        SwapETHForExactTokens(SwapETHForExactTokensCall),
        SwapExactETHForTokens(SwapExactETHForTokensCall),
        SwapExactTokensForETH(SwapExactTokensForETHCall),
        SwapExactTokensForTokens(SwapExactTokensForTokensCall),
        SwapTokensForExactETH(SwapTokensForExactETHCall),
        SwapTokensForExactTokens(SwapTokensForExactTokensCall),
    }
    impl ethers::core::abi::AbiDecode for IPancakeRouter01Calls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) = <WethCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(IPancakeRouter01Calls::Weth(decoded));
            }
            if let Ok(decoded) =
                <AddLiquidityCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakeRouter01Calls::AddLiquidity(decoded));
            }
            if let Ok(decoded) =
                <AddLiquidityETHCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakeRouter01Calls::AddLiquidityETH(decoded));
            }
            if let Ok(decoded) =
                <FactoryCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakeRouter01Calls::Factory(decoded));
            }
            if let Ok(decoded) =
                <GetAmountInCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakeRouter01Calls::GetAmountIn(decoded));
            }
            if let Ok(decoded) =
                <GetAmountOutCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakeRouter01Calls::GetAmountOut(decoded));
            }
            if let Ok(decoded) =
                <GetAmountsInCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakeRouter01Calls::GetAmountsIn(decoded));
            }
            if let Ok(decoded) =
                <GetAmountsOutCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakeRouter01Calls::GetAmountsOut(decoded));
            }
            if let Ok(decoded) = <QuoteCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakeRouter01Calls::Quote(decoded));
            }
            if let Ok(decoded) =
                <RemoveLiquidityCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakeRouter01Calls::RemoveLiquidity(decoded));
            }
            if let Ok(decoded) =
                <RemoveLiquidityETHCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakeRouter01Calls::RemoveLiquidityETH(decoded));
            }
            if let Ok(decoded) =
                <RemoveLiquidityETHWithPermitCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(IPancakeRouter01Calls::RemoveLiquidityETHWithPermit(decoded));
            }
            if let Ok(decoded) =
                <RemoveLiquidityWithPermitCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(IPancakeRouter01Calls::RemoveLiquidityWithPermit(decoded));
            }
            if let Ok(decoded) =
                <SwapETHForExactTokensCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakeRouter01Calls::SwapETHForExactTokens(decoded));
            }
            if let Ok(decoded) =
                <SwapExactETHForTokensCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakeRouter01Calls::SwapExactETHForTokens(decoded));
            }
            if let Ok(decoded) =
                <SwapExactTokensForETHCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakeRouter01Calls::SwapExactTokensForETH(decoded));
            }
            if let Ok(decoded) =
                <SwapExactTokensForTokensCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(IPancakeRouter01Calls::SwapExactTokensForTokens(decoded));
            }
            if let Ok(decoded) =
                <SwapTokensForExactETHCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakeRouter01Calls::SwapTokensForExactETH(decoded));
            }
            if let Ok(decoded) =
                <SwapTokensForExactTokensCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(IPancakeRouter01Calls::SwapTokensForExactTokens(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for IPancakeRouter01Calls {
        fn encode(self) -> Vec<u8> {
            match self {
                IPancakeRouter01Calls::Weth(element) => element.encode(),
                IPancakeRouter01Calls::AddLiquidity(element) => element.encode(),
                IPancakeRouter01Calls::AddLiquidityETH(element) => element.encode(),
                IPancakeRouter01Calls::Factory(element) => element.encode(),
                IPancakeRouter01Calls::GetAmountIn(element) => element.encode(),
                IPancakeRouter01Calls::GetAmountOut(element) => element.encode(),
                IPancakeRouter01Calls::GetAmountsIn(element) => element.encode(),
                IPancakeRouter01Calls::GetAmountsOut(element) => element.encode(),
                IPancakeRouter01Calls::Quote(element) => element.encode(),
                IPancakeRouter01Calls::RemoveLiquidity(element) => element.encode(),
                IPancakeRouter01Calls::RemoveLiquidityETH(element) => element.encode(),
                IPancakeRouter01Calls::RemoveLiquidityETHWithPermit(element) => element.encode(),
                IPancakeRouter01Calls::RemoveLiquidityWithPermit(element) => element.encode(),
                IPancakeRouter01Calls::SwapETHForExactTokens(element) => element.encode(),
                IPancakeRouter01Calls::SwapExactETHForTokens(element) => element.encode(),
                IPancakeRouter01Calls::SwapExactTokensForETH(element) => element.encode(),
                IPancakeRouter01Calls::SwapExactTokensForTokens(element) => element.encode(),
                IPancakeRouter01Calls::SwapTokensForExactETH(element) => element.encode(),
                IPancakeRouter01Calls::SwapTokensForExactTokens(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for IPancakeRouter01Calls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                IPancakeRouter01Calls::Weth(element) => element.fmt(f),
                IPancakeRouter01Calls::AddLiquidity(element) => element.fmt(f),
                IPancakeRouter01Calls::AddLiquidityETH(element) => element.fmt(f),
                IPancakeRouter01Calls::Factory(element) => element.fmt(f),
                IPancakeRouter01Calls::GetAmountIn(element) => element.fmt(f),
                IPancakeRouter01Calls::GetAmountOut(element) => element.fmt(f),
                IPancakeRouter01Calls::GetAmountsIn(element) => element.fmt(f),
                IPancakeRouter01Calls::GetAmountsOut(element) => element.fmt(f),
                IPancakeRouter01Calls::Quote(element) => element.fmt(f),
                IPancakeRouter01Calls::RemoveLiquidity(element) => element.fmt(f),
                IPancakeRouter01Calls::RemoveLiquidityETH(element) => element.fmt(f),
                IPancakeRouter01Calls::RemoveLiquidityETHWithPermit(element) => element.fmt(f),
                IPancakeRouter01Calls::RemoveLiquidityWithPermit(element) => element.fmt(f),
                IPancakeRouter01Calls::SwapETHForExactTokens(element) => element.fmt(f),
                IPancakeRouter01Calls::SwapExactETHForTokens(element) => element.fmt(f),
                IPancakeRouter01Calls::SwapExactTokensForETH(element) => element.fmt(f),
                IPancakeRouter01Calls::SwapExactTokensForTokens(element) => element.fmt(f),
                IPancakeRouter01Calls::SwapTokensForExactETH(element) => element.fmt(f),
                IPancakeRouter01Calls::SwapTokensForExactTokens(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<WethCall> for IPancakeRouter01Calls {
        fn from(var: WethCall) -> Self {
            IPancakeRouter01Calls::Weth(var)
        }
    }
    impl ::std::convert::From<AddLiquidityCall> for IPancakeRouter01Calls {
        fn from(var: AddLiquidityCall) -> Self {
            IPancakeRouter01Calls::AddLiquidity(var)
        }
    }
    impl ::std::convert::From<AddLiquidityETHCall> for IPancakeRouter01Calls {
        fn from(var: AddLiquidityETHCall) -> Self {
            IPancakeRouter01Calls::AddLiquidityETH(var)
        }
    }
    impl ::std::convert::From<FactoryCall> for IPancakeRouter01Calls {
        fn from(var: FactoryCall) -> Self {
            IPancakeRouter01Calls::Factory(var)
        }
    }
    impl ::std::convert::From<GetAmountInCall> for IPancakeRouter01Calls {
        fn from(var: GetAmountInCall) -> Self {
            IPancakeRouter01Calls::GetAmountIn(var)
        }
    }
    impl ::std::convert::From<GetAmountOutCall> for IPancakeRouter01Calls {
        fn from(var: GetAmountOutCall) -> Self {
            IPancakeRouter01Calls::GetAmountOut(var)
        }
    }
    impl ::std::convert::From<GetAmountsInCall> for IPancakeRouter01Calls {
        fn from(var: GetAmountsInCall) -> Self {
            IPancakeRouter01Calls::GetAmountsIn(var)
        }
    }
    impl ::std::convert::From<GetAmountsOutCall> for IPancakeRouter01Calls {
        fn from(var: GetAmountsOutCall) -> Self {
            IPancakeRouter01Calls::GetAmountsOut(var)
        }
    }
    impl ::std::convert::From<QuoteCall> for IPancakeRouter01Calls {
        fn from(var: QuoteCall) -> Self {
            IPancakeRouter01Calls::Quote(var)
        }
    }
    impl ::std::convert::From<RemoveLiquidityCall> for IPancakeRouter01Calls {
        fn from(var: RemoveLiquidityCall) -> Self {
            IPancakeRouter01Calls::RemoveLiquidity(var)
        }
    }
    impl ::std::convert::From<RemoveLiquidityETHCall> for IPancakeRouter01Calls {
        fn from(var: RemoveLiquidityETHCall) -> Self {
            IPancakeRouter01Calls::RemoveLiquidityETH(var)
        }
    }
    impl ::std::convert::From<RemoveLiquidityETHWithPermitCall> for IPancakeRouter01Calls {
        fn from(var: RemoveLiquidityETHWithPermitCall) -> Self {
            IPancakeRouter01Calls::RemoveLiquidityETHWithPermit(var)
        }
    }
    impl ::std::convert::From<RemoveLiquidityWithPermitCall> for IPancakeRouter01Calls {
        fn from(var: RemoveLiquidityWithPermitCall) -> Self {
            IPancakeRouter01Calls::RemoveLiquidityWithPermit(var)
        }
    }
    impl ::std::convert::From<SwapETHForExactTokensCall> for IPancakeRouter01Calls {
        fn from(var: SwapETHForExactTokensCall) -> Self {
            IPancakeRouter01Calls::SwapETHForExactTokens(var)
        }
    }
    impl ::std::convert::From<SwapExactETHForTokensCall> for IPancakeRouter01Calls {
        fn from(var: SwapExactETHForTokensCall) -> Self {
            IPancakeRouter01Calls::SwapExactETHForTokens(var)
        }
    }
    impl ::std::convert::From<SwapExactTokensForETHCall> for IPancakeRouter01Calls {
        fn from(var: SwapExactTokensForETHCall) -> Self {
            IPancakeRouter01Calls::SwapExactTokensForETH(var)
        }
    }
    impl ::std::convert::From<SwapExactTokensForTokensCall> for IPancakeRouter01Calls {
        fn from(var: SwapExactTokensForTokensCall) -> Self {
            IPancakeRouter01Calls::SwapExactTokensForTokens(var)
        }
    }
    impl ::std::convert::From<SwapTokensForExactETHCall> for IPancakeRouter01Calls {
        fn from(var: SwapTokensForExactETHCall) -> Self {
            IPancakeRouter01Calls::SwapTokensForExactETH(var)
        }
    }
    impl ::std::convert::From<SwapTokensForExactTokensCall> for IPancakeRouter01Calls {
        fn from(var: SwapTokensForExactTokensCall) -> Self {
            IPancakeRouter01Calls::SwapTokensForExactTokens(var)
        }
    }
    #[doc = "Container type for all return fields from the `WETH` function with signature `WETH()` and selector `[173, 92, 70, 72]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct WethReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `addLiquidity` function with signature `addLiquidity(address,address,uint256,uint256,uint256,uint256,address,uint256)` and selector `[232, 227, 55, 0]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct AddLiquidityReturn {
        pub amount_a: ethers::core::types::U256,
        pub amount_b: ethers::core::types::U256,
        pub liquidity: ethers::core::types::U256,
    }
    #[doc = "Container type for all return fields from the `addLiquidityETH` function with signature `addLiquidityETH(address,uint256,uint256,uint256,address,uint256)` and selector `[243, 5, 215, 25]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct AddLiquidityETHReturn {
        pub amount_token: ethers::core::types::U256,
        pub amount_eth: ethers::core::types::U256,
        pub liquidity: ethers::core::types::U256,
    }
    #[doc = "Container type for all return fields from the `factory` function with signature `factory()` and selector `[196, 90, 1, 85]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct FactoryReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `getAmountIn` function with signature `getAmountIn(uint256,uint256,uint256)` and selector `[133, 248, 194, 89]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct GetAmountInReturn {
        pub amount_in: ethers::core::types::U256,
    }
    #[doc = "Container type for all return fields from the `getAmountOut` function with signature `getAmountOut(uint256,uint256,uint256)` and selector `[5, 77, 80, 212]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct GetAmountOutReturn {
        pub amount_out: ethers::core::types::U256,
    }
    #[doc = "Container type for all return fields from the `getAmountsIn` function with signature `getAmountsIn(uint256,address[])` and selector `[31, 0, 202, 116]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct GetAmountsInReturn {
        pub amounts: ::std::vec::Vec<ethers::core::types::U256>,
    }
    #[doc = "Container type for all return fields from the `getAmountsOut` function with signature `getAmountsOut(uint256,address[])` and selector `[208, 108, 166, 31]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct GetAmountsOutReturn {
        pub amounts: ::std::vec::Vec<ethers::core::types::U256>,
    }
    #[doc = "Container type for all return fields from the `quote` function with signature `quote(uint256,uint256,uint256)` and selector `[173, 97, 93, 236]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct QuoteReturn {
        pub amount_b: ethers::core::types::U256,
    }
    #[doc = "Container type for all return fields from the `removeLiquidity` function with signature `removeLiquidity(address,address,uint256,uint256,uint256,address,uint256)` and selector `[186, 162, 171, 222]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct RemoveLiquidityReturn {
        pub amount_a: ethers::core::types::U256,
        pub amount_b: ethers::core::types::U256,
    }
    #[doc = "Container type for all return fields from the `removeLiquidityETH` function with signature `removeLiquidityETH(address,uint256,uint256,uint256,address,uint256)` and selector `[2, 117, 28, 236]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct RemoveLiquidityETHReturn {
        pub amount_token: ethers::core::types::U256,
        pub amount_eth: ethers::core::types::U256,
    }
    #[doc = "Container type for all return fields from the `removeLiquidityETHWithPermit` function with signature `removeLiquidityETHWithPermit(address,uint256,uint256,uint256,address,uint256,bool,uint8,bytes32,bytes32)` and selector `[222, 217, 56, 42]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct RemoveLiquidityETHWithPermitReturn {
        pub amount_token: ethers::core::types::U256,
        pub amount_eth: ethers::core::types::U256,
    }
    #[doc = "Container type for all return fields from the `removeLiquidityWithPermit` function with signature `removeLiquidityWithPermit(address,address,uint256,uint256,uint256,address,uint256,bool,uint8,bytes32,bytes32)` and selector `[33, 149, 153, 92]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct RemoveLiquidityWithPermitReturn {
        pub amount_a: ethers::core::types::U256,
        pub amount_b: ethers::core::types::U256,
    }
    #[doc = "Container type for all return fields from the `swapETHForExactTokens` function with signature `swapETHForExactTokens(uint256,address[],address,uint256)` and selector `[251, 59, 219, 65]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct SwapETHForExactTokensReturn {
        pub amounts: ::std::vec::Vec<ethers::core::types::U256>,
    }
    #[doc = "Container type for all return fields from the `swapExactETHForTokens` function with signature `swapExactETHForTokens(uint256,address[],address,uint256)` and selector `[127, 243, 106, 181]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct SwapExactETHForTokensReturn {
        pub amounts: ::std::vec::Vec<ethers::core::types::U256>,
    }
    #[doc = "Container type for all return fields from the `swapExactTokensForETH` function with signature `swapExactTokensForETH(uint256,uint256,address[],address,uint256)` and selector `[24, 203, 175, 229]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct SwapExactTokensForETHReturn {
        pub amounts: ::std::vec::Vec<ethers::core::types::U256>,
    }
    #[doc = "Container type for all return fields from the `swapExactTokensForTokens` function with signature `swapExactTokensForTokens(uint256,uint256,address[],address,uint256)` and selector `[56, 237, 23, 57]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct SwapExactTokensForTokensReturn {
        pub amounts: ::std::vec::Vec<ethers::core::types::U256>,
    }
    #[doc = "Container type for all return fields from the `swapTokensForExactETH` function with signature `swapTokensForExactETH(uint256,uint256,address[],address,uint256)` and selector `[74, 37, 217, 74]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct SwapTokensForExactETHReturn {
        pub amounts: ::std::vec::Vec<ethers::core::types::U256>,
    }
    #[doc = "Container type for all return fields from the `swapTokensForExactTokens` function with signature `swapTokensForExactTokens(uint256,uint256,address[],address,uint256)` and selector `[136, 3, 219, 238]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct SwapTokensForExactTokensReturn {
        pub amounts: ::std::vec::Vec<ethers::core::types::U256>,
    }
}
