pub use i_pancake_pair::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod i_pancake_pair {
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
    #[doc = "IPancakePair was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"owner\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"spender\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Approval\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"amount0\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"amount1\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"Burn\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"amount0\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"amount1\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Mint\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"amount0In\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"amount1In\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"amount0Out\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"amount1Out\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"Swap\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"uint112\",\"name\":\"reserve0\",\"type\":\"uint112\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint112\",\"name\":\"reserve1\",\"type\":\"uint112\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Sync\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Transfer\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"DOMAIN_SEPARATOR\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"pure\",\"type\":\"function\",\"name\":\"MINIMUM_LIQUIDITY\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"pure\",\"type\":\"function\",\"name\":\"PERMIT_TYPEHASH\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"owner\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"spender\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"allowance\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"spender\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"approve\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"owner\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"balanceOf\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"burn\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"amount0\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount1\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"pure\",\"type\":\"function\",\"name\":\"decimals\",\"outputs\":[{\"internalType\":\"uint8\",\"name\":\"\",\"type\":\"uint8\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"factory\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getReserves\",\"outputs\":[{\"internalType\":\"uint112\",\"name\":\"reserve0\",\"type\":\"uint112\",\"components\":[]},{\"internalType\":\"uint112\",\"name\":\"reserve1\",\"type\":\"uint112\",\"components\":[]},{\"internalType\":\"uint32\",\"name\":\"blockTimestampLast\",\"type\":\"uint32\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"initialize\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"kLast\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"mint\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"liquidity\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"pure\",\"type\":\"function\",\"name\":\"name\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"owner\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"nonces\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"owner\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"spender\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"deadline\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint8\",\"name\":\"v\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"r\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"s\",\"type\":\"bytes32\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"permit\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"price0CumulativeLast\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"price1CumulativeLast\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"skim\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"amount0Out\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount1Out\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"swap\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"pure\",\"type\":\"function\",\"name\":\"symbol\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"sync\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"token0\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"token1\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"totalSupply\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transfer\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transferFrom\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static IPANCAKEPAIR_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    pub struct IPancakePair<M>(ethers::contract::Contract<M>);
    impl<M> Clone for IPancakePair<M> {
        fn clone(&self) -> Self {
            IPancakePair(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for IPancakePair<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for IPancakePair<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(IPancakePair))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> IPancakePair<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), IPANCAKEPAIR_ABI.clone(), client).into()
        }
        #[doc = "Calls the contract's `DOMAIN_SEPARATOR` (0x3644e515) function"]
        pub fn domain_separator(&self) -> ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([54, 68, 229, 21], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `MINIMUM_LIQUIDITY` (0xba9a7a56) function"]
        pub fn minimum_liquidity(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([186, 154, 122, 86], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `PERMIT_TYPEHASH` (0x30adf81f) function"]
        pub fn permit_typehash(&self) -> ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([48, 173, 248, 31], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `allowance` (0xdd62ed3e) function"]
        pub fn allowance(
            &self,
            owner: ethers::core::types::Address,
            spender: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([221, 98, 237, 62], (owner, spender))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `approve` (0x095ea7b3) function"]
        pub fn approve(
            &self,
            spender: ethers::core::types::Address,
            value: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([9, 94, 167, 179], (spender, value))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `balanceOf` (0x70a08231) function"]
        pub fn balance_of(
            &self,
            owner: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([112, 160, 130, 49], owner)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `burn` (0x89afcb44) function"]
        pub fn burn(
            &self,
            to: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<
            M,
            (ethers::core::types::U256, ethers::core::types::U256),
        > {
            self.0
                .method_hash([137, 175, 203, 68], to)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `decimals` (0x313ce567) function"]
        pub fn decimals(&self) -> ethers::contract::builders::ContractCall<M, u8> {
            self.0
                .method_hash([49, 60, 229, 103], ())
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
        #[doc = "Calls the contract's `getReserves` (0x0902f1ac) function"]
        pub fn get_reserves(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, (u128, u128, u32)> {
            self.0
                .method_hash([9, 2, 241, 172], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `initialize` (0x485cc955) function"]
        pub fn initialize(
            &self,
            p0: ethers::core::types::Address,
            p1: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([72, 92, 201, 85], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `kLast` (0x7464fc3d) function"]
        pub fn k_last(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([116, 100, 252, 61], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `mint` (0x6a627842) function"]
        pub fn mint(
            &self,
            to: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([106, 98, 120, 66], to)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `name` (0x06fdde03) function"]
        pub fn name(&self) -> ethers::contract::builders::ContractCall<M, String> {
            self.0
                .method_hash([6, 253, 222, 3], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `nonces` (0x7ecebe00) function"]
        pub fn nonces(
            &self,
            owner: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([126, 206, 190, 0], owner)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `permit` (0xd505accf) function"]
        pub fn permit(
            &self,
            owner: ethers::core::types::Address,
            spender: ethers::core::types::Address,
            value: ethers::core::types::U256,
            deadline: ethers::core::types::U256,
            v: u8,
            r: [u8; 32],
            s: [u8; 32],
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [213, 5, 172, 207],
                    (owner, spender, value, deadline, v, r, s),
                )
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `price0CumulativeLast` (0x5909c0d5) function"]
        pub fn price_0_cumulative_last(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([89, 9, 192, 213], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `price1CumulativeLast` (0x5a3d5493) function"]
        pub fn price_1_cumulative_last(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([90, 61, 84, 147], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `skim` (0xbc25cf77) function"]
        pub fn skim(
            &self,
            to: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([188, 37, 207, 119], to)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `swap` (0x022c0d9f) function"]
        pub fn swap(
            &self,
            amount_0_out: ethers::core::types::U256,
            amount_1_out: ethers::core::types::U256,
            to: ethers::core::types::Address,
            data: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([2, 44, 13, 159], (amount_0_out, amount_1_out, to, data))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `symbol` (0x95d89b41) function"]
        pub fn symbol(&self) -> ethers::contract::builders::ContractCall<M, String> {
            self.0
                .method_hash([149, 216, 155, 65], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `sync` (0xfff6cae9) function"]
        pub fn sync(&self) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([255, 246, 202, 233], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `token0` (0x0dfe1681) function"]
        pub fn token_0(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([13, 254, 22, 129], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `token1` (0xd21220a7) function"]
        pub fn token_1(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([210, 18, 32, 167], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `totalSupply` (0x18160ddd) function"]
        pub fn total_supply(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([24, 22, 13, 221], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `transfer` (0xa9059cbb) function"]
        pub fn transfer(
            &self,
            to: ethers::core::types::Address,
            value: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([169, 5, 156, 187], (to, value))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `transferFrom` (0x23b872dd) function"]
        pub fn transfer_from(
            &self,
            from: ethers::core::types::Address,
            to: ethers::core::types::Address,
            value: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([35, 184, 114, 221], (from, to, value))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Gets the contract's `Approval` event"]
        pub fn approval_filter(&self) -> ethers::contract::builders::Event<M, ApprovalFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Burn` event"]
        pub fn burn_filter(&self) -> ethers::contract::builders::Event<M, BurnFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Mint` event"]
        pub fn mint_filter(&self) -> ethers::contract::builders::Event<M, MintFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Swap` event"]
        pub fn swap_filter(&self) -> ethers::contract::builders::Event<M, SwapFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Sync` event"]
        pub fn sync_filter(&self) -> ethers::contract::builders::Event<M, SyncFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Transfer` event"]
        pub fn transfer_filter(&self) -> ethers::contract::builders::Event<M, TransferFilter> {
            self.0.event()
        }
        #[doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(&self) -> ethers::contract::builders::Event<M, IPancakePairEvents> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for IPancakePair<M> {
        fn from(contract: ethers::contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "Approval", abi = "Approval(address,address,uint256)")]
    pub struct ApprovalFilter {
        #[ethevent(indexed)]
        pub owner: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub spender: ethers::core::types::Address,
        pub value: ethers::core::types::U256,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "Burn", abi = "Burn(address,uint256,uint256,address)")]
    pub struct BurnFilter {
        #[ethevent(indexed)]
        pub sender: ethers::core::types::Address,
        pub amount_0: ethers::core::types::U256,
        pub amount_1: ethers::core::types::U256,
        #[ethevent(indexed)]
        pub to: ethers::core::types::Address,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "Mint", abi = "Mint(address,uint256,uint256)")]
    pub struct MintFilter {
        #[ethevent(indexed)]
        pub sender: ethers::core::types::Address,
        pub amount_0: ethers::core::types::U256,
        pub amount_1: ethers::core::types::U256,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(
        name = "Swap",
        abi = "Swap(address,uint256,uint256,uint256,uint256,address)"
    )]
    pub struct SwapFilter {
        #[ethevent(indexed)]
        pub sender: ethers::core::types::Address,
        pub amount_0_in: ethers::core::types::U256,
        pub amount_1_in: ethers::core::types::U256,
        pub amount_0_out: ethers::core::types::U256,
        pub amount_1_out: ethers::core::types::U256,
        #[ethevent(indexed)]
        pub to: ethers::core::types::Address,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "Sync", abi = "Sync(uint112,uint112)")]
    pub struct SyncFilter {
        pub reserve_0: u128,
        pub reserve_1: u128,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "Transfer", abi = "Transfer(address,address,uint256)")]
    pub struct TransferFilter {
        #[ethevent(indexed)]
        pub from: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub to: ethers::core::types::Address,
        pub value: ethers::core::types::U256,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum IPancakePairEvents {
        ApprovalFilter(ApprovalFilter),
        BurnFilter(BurnFilter),
        MintFilter(MintFilter),
        SwapFilter(SwapFilter),
        SyncFilter(SyncFilter),
        TransferFilter(TransferFilter),
    }
    impl ethers::contract::EthLogDecode for IPancakePairEvents {
        fn decode_log(
            log: &ethers::core::abi::RawLog,
        ) -> ::std::result::Result<Self, ethers::core::abi::Error>
        where
            Self: Sized,
        {
            if let Ok(decoded) = ApprovalFilter::decode_log(log) {
                return Ok(IPancakePairEvents::ApprovalFilter(decoded));
            }
            if let Ok(decoded) = BurnFilter::decode_log(log) {
                return Ok(IPancakePairEvents::BurnFilter(decoded));
            }
            if let Ok(decoded) = MintFilter::decode_log(log) {
                return Ok(IPancakePairEvents::MintFilter(decoded));
            }
            if let Ok(decoded) = SwapFilter::decode_log(log) {
                return Ok(IPancakePairEvents::SwapFilter(decoded));
            }
            if let Ok(decoded) = SyncFilter::decode_log(log) {
                return Ok(IPancakePairEvents::SyncFilter(decoded));
            }
            if let Ok(decoded) = TransferFilter::decode_log(log) {
                return Ok(IPancakePairEvents::TransferFilter(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::std::fmt::Display for IPancakePairEvents {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                IPancakePairEvents::ApprovalFilter(element) => element.fmt(f),
                IPancakePairEvents::BurnFilter(element) => element.fmt(f),
                IPancakePairEvents::MintFilter(element) => element.fmt(f),
                IPancakePairEvents::SwapFilter(element) => element.fmt(f),
                IPancakePairEvents::SyncFilter(element) => element.fmt(f),
                IPancakePairEvents::TransferFilter(element) => element.fmt(f),
            }
        }
    }
    #[doc = "Container type for all input parameters for the `DOMAIN_SEPARATOR` function with signature `DOMAIN_SEPARATOR()` and selector `[54, 68, 229, 21]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "DOMAIN_SEPARATOR", abi = "DOMAIN_SEPARATOR()")]
    pub struct DomainSeparatorCall;
    #[doc = "Container type for all input parameters for the `MINIMUM_LIQUIDITY` function with signature `MINIMUM_LIQUIDITY()` and selector `[186, 154, 122, 86]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "MINIMUM_LIQUIDITY", abi = "MINIMUM_LIQUIDITY()")]
    pub struct MinimumLiquidityCall;
    #[doc = "Container type for all input parameters for the `PERMIT_TYPEHASH` function with signature `PERMIT_TYPEHASH()` and selector `[48, 173, 248, 31]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "PERMIT_TYPEHASH", abi = "PERMIT_TYPEHASH()")]
    pub struct PermitTypehashCall;
    #[doc = "Container type for all input parameters for the `allowance` function with signature `allowance(address,address)` and selector `[221, 98, 237, 62]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "allowance", abi = "allowance(address,address)")]
    pub struct AllowanceCall {
        pub owner: ethers::core::types::Address,
        pub spender: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `approve` function with signature `approve(address,uint256)` and selector `[9, 94, 167, 179]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "approve", abi = "approve(address,uint256)")]
    pub struct ApproveCall {
        pub spender: ethers::core::types::Address,
        pub value: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `balanceOf` function with signature `balanceOf(address)` and selector `[112, 160, 130, 49]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "balanceOf", abi = "balanceOf(address)")]
    pub struct BalanceOfCall {
        pub owner: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `burn` function with signature `burn(address)` and selector `[137, 175, 203, 68]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "burn", abi = "burn(address)")]
    pub struct BurnCall {
        pub to: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `decimals` function with signature `decimals()` and selector `[49, 60, 229, 103]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "decimals", abi = "decimals()")]
    pub struct DecimalsCall;
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
    #[doc = "Container type for all input parameters for the `getReserves` function with signature `getReserves()` and selector `[9, 2, 241, 172]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "getReserves", abi = "getReserves()")]
    pub struct GetReservesCall;
    #[doc = "Container type for all input parameters for the `initialize` function with signature `initialize(address,address)` and selector `[72, 92, 201, 85]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "initialize", abi = "initialize(address,address)")]
    pub struct InitializeCall(
        pub ethers::core::types::Address,
        pub ethers::core::types::Address,
    );
    #[doc = "Container type for all input parameters for the `kLast` function with signature `kLast()` and selector `[116, 100, 252, 61]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "kLast", abi = "kLast()")]
    pub struct KlastCall;
    #[doc = "Container type for all input parameters for the `mint` function with signature `mint(address)` and selector `[106, 98, 120, 66]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "mint", abi = "mint(address)")]
    pub struct MintCall {
        pub to: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `name` function with signature `name()` and selector `[6, 253, 222, 3]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "name", abi = "name()")]
    pub struct NameCall;
    #[doc = "Container type for all input parameters for the `nonces` function with signature `nonces(address)` and selector `[126, 206, 190, 0]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "nonces", abi = "nonces(address)")]
    pub struct NoncesCall {
        pub owner: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `permit` function with signature `permit(address,address,uint256,uint256,uint8,bytes32,bytes32)` and selector `[213, 5, 172, 207]`"]
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
        name = "permit",
        abi = "permit(address,address,uint256,uint256,uint8,bytes32,bytes32)"
    )]
    pub struct PermitCall {
        pub owner: ethers::core::types::Address,
        pub spender: ethers::core::types::Address,
        pub value: ethers::core::types::U256,
        pub deadline: ethers::core::types::U256,
        pub v: u8,
        pub r: [u8; 32],
        pub s: [u8; 32],
    }
    #[doc = "Container type for all input parameters for the `price0CumulativeLast` function with signature `price0CumulativeLast()` and selector `[89, 9, 192, 213]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "price0CumulativeLast", abi = "price0CumulativeLast()")]
    pub struct Price0CumulativeLastCall;
    #[doc = "Container type for all input parameters for the `price1CumulativeLast` function with signature `price1CumulativeLast()` and selector `[90, 61, 84, 147]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "price1CumulativeLast", abi = "price1CumulativeLast()")]
    pub struct Price1CumulativeLastCall;
    #[doc = "Container type for all input parameters for the `skim` function with signature `skim(address)` and selector `[188, 37, 207, 119]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "skim", abi = "skim(address)")]
    pub struct SkimCall {
        pub to: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `swap` function with signature `swap(uint256,uint256,address,bytes)` and selector `[2, 44, 13, 159]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "swap", abi = "swap(uint256,uint256,address,bytes)")]
    pub struct SwapCall {
        pub amount_0_out: ethers::core::types::U256,
        pub amount_1_out: ethers::core::types::U256,
        pub to: ethers::core::types::Address,
        pub data: ethers::core::types::Bytes,
    }
    #[doc = "Container type for all input parameters for the `symbol` function with signature `symbol()` and selector `[149, 216, 155, 65]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "symbol", abi = "symbol()")]
    pub struct SymbolCall;
    #[doc = "Container type for all input parameters for the `sync` function with signature `sync()` and selector `[255, 246, 202, 233]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "sync", abi = "sync()")]
    pub struct SyncCall;
    #[doc = "Container type for all input parameters for the `token0` function with signature `token0()` and selector `[13, 254, 22, 129]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "token0", abi = "token0()")]
    pub struct Token0Call;
    #[doc = "Container type for all input parameters for the `token1` function with signature `token1()` and selector `[210, 18, 32, 167]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "token1", abi = "token1()")]
    pub struct Token1Call;
    #[doc = "Container type for all input parameters for the `totalSupply` function with signature `totalSupply()` and selector `[24, 22, 13, 221]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "totalSupply", abi = "totalSupply()")]
    pub struct TotalSupplyCall;
    #[doc = "Container type for all input parameters for the `transfer` function with signature `transfer(address,uint256)` and selector `[169, 5, 156, 187]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "transfer", abi = "transfer(address,uint256)")]
    pub struct TransferCall {
        pub to: ethers::core::types::Address,
        pub value: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `transferFrom` function with signature `transferFrom(address,address,uint256)` and selector `[35, 184, 114, 221]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "transferFrom", abi = "transferFrom(address,address,uint256)")]
    pub struct TransferFromCall {
        pub from: ethers::core::types::Address,
        pub to: ethers::core::types::Address,
        pub value: ethers::core::types::U256,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum IPancakePairCalls {
        DomainSeparator(DomainSeparatorCall),
        MinimumLiquidity(MinimumLiquidityCall),
        PermitTypehash(PermitTypehashCall),
        Allowance(AllowanceCall),
        Approve(ApproveCall),
        BalanceOf(BalanceOfCall),
        Burn(BurnCall),
        Decimals(DecimalsCall),
        Factory(FactoryCall),
        GetReserves(GetReservesCall),
        Initialize(InitializeCall),
        Klast(KlastCall),
        Mint(MintCall),
        Name(NameCall),
        Nonces(NoncesCall),
        Permit(PermitCall),
        Price0CumulativeLast(Price0CumulativeLastCall),
        Price1CumulativeLast(Price1CumulativeLastCall),
        Skim(SkimCall),
        Swap(SwapCall),
        Symbol(SymbolCall),
        Sync(SyncCall),
        Token0(Token0Call),
        Token1(Token1Call),
        TotalSupply(TotalSupplyCall),
        Transfer(TransferCall),
        TransferFrom(TransferFromCall),
    }
    impl ethers::core::abi::AbiDecode for IPancakePairCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <DomainSeparatorCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakePairCalls::DomainSeparator(decoded));
            }
            if let Ok(decoded) =
                <MinimumLiquidityCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakePairCalls::MinimumLiquidity(decoded));
            }
            if let Ok(decoded) =
                <PermitTypehashCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakePairCalls::PermitTypehash(decoded));
            }
            if let Ok(decoded) =
                <AllowanceCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakePairCalls::Allowance(decoded));
            }
            if let Ok(decoded) =
                <ApproveCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakePairCalls::Approve(decoded));
            }
            if let Ok(decoded) =
                <BalanceOfCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakePairCalls::BalanceOf(decoded));
            }
            if let Ok(decoded) = <BurnCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(IPancakePairCalls::Burn(decoded));
            }
            if let Ok(decoded) =
                <DecimalsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakePairCalls::Decimals(decoded));
            }
            if let Ok(decoded) =
                <FactoryCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakePairCalls::Factory(decoded));
            }
            if let Ok(decoded) =
                <GetReservesCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakePairCalls::GetReserves(decoded));
            }
            if let Ok(decoded) =
                <InitializeCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakePairCalls::Initialize(decoded));
            }
            if let Ok(decoded) = <KlastCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakePairCalls::Klast(decoded));
            }
            if let Ok(decoded) = <MintCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(IPancakePairCalls::Mint(decoded));
            }
            if let Ok(decoded) = <NameCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(IPancakePairCalls::Name(decoded));
            }
            if let Ok(decoded) = <NoncesCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakePairCalls::Nonces(decoded));
            }
            if let Ok(decoded) = <PermitCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakePairCalls::Permit(decoded));
            }
            if let Ok(decoded) =
                <Price0CumulativeLastCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakePairCalls::Price0CumulativeLast(decoded));
            }
            if let Ok(decoded) =
                <Price1CumulativeLastCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakePairCalls::Price1CumulativeLast(decoded));
            }
            if let Ok(decoded) = <SkimCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(IPancakePairCalls::Skim(decoded));
            }
            if let Ok(decoded) = <SwapCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(IPancakePairCalls::Swap(decoded));
            }
            if let Ok(decoded) = <SymbolCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakePairCalls::Symbol(decoded));
            }
            if let Ok(decoded) = <SyncCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(IPancakePairCalls::Sync(decoded));
            }
            if let Ok(decoded) = <Token0Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakePairCalls::Token0(decoded));
            }
            if let Ok(decoded) = <Token1Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakePairCalls::Token1(decoded));
            }
            if let Ok(decoded) =
                <TotalSupplyCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakePairCalls::TotalSupply(decoded));
            }
            if let Ok(decoded) =
                <TransferCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakePairCalls::Transfer(decoded));
            }
            if let Ok(decoded) =
                <TransferFromCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakePairCalls::TransferFrom(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for IPancakePairCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                IPancakePairCalls::DomainSeparator(element) => element.encode(),
                IPancakePairCalls::MinimumLiquidity(element) => element.encode(),
                IPancakePairCalls::PermitTypehash(element) => element.encode(),
                IPancakePairCalls::Allowance(element) => element.encode(),
                IPancakePairCalls::Approve(element) => element.encode(),
                IPancakePairCalls::BalanceOf(element) => element.encode(),
                IPancakePairCalls::Burn(element) => element.encode(),
                IPancakePairCalls::Decimals(element) => element.encode(),
                IPancakePairCalls::Factory(element) => element.encode(),
                IPancakePairCalls::GetReserves(element) => element.encode(),
                IPancakePairCalls::Initialize(element) => element.encode(),
                IPancakePairCalls::Klast(element) => element.encode(),
                IPancakePairCalls::Mint(element) => element.encode(),
                IPancakePairCalls::Name(element) => element.encode(),
                IPancakePairCalls::Nonces(element) => element.encode(),
                IPancakePairCalls::Permit(element) => element.encode(),
                IPancakePairCalls::Price0CumulativeLast(element) => element.encode(),
                IPancakePairCalls::Price1CumulativeLast(element) => element.encode(),
                IPancakePairCalls::Skim(element) => element.encode(),
                IPancakePairCalls::Swap(element) => element.encode(),
                IPancakePairCalls::Symbol(element) => element.encode(),
                IPancakePairCalls::Sync(element) => element.encode(),
                IPancakePairCalls::Token0(element) => element.encode(),
                IPancakePairCalls::Token1(element) => element.encode(),
                IPancakePairCalls::TotalSupply(element) => element.encode(),
                IPancakePairCalls::Transfer(element) => element.encode(),
                IPancakePairCalls::TransferFrom(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for IPancakePairCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                IPancakePairCalls::DomainSeparator(element) => element.fmt(f),
                IPancakePairCalls::MinimumLiquidity(element) => element.fmt(f),
                IPancakePairCalls::PermitTypehash(element) => element.fmt(f),
                IPancakePairCalls::Allowance(element) => element.fmt(f),
                IPancakePairCalls::Approve(element) => element.fmt(f),
                IPancakePairCalls::BalanceOf(element) => element.fmt(f),
                IPancakePairCalls::Burn(element) => element.fmt(f),
                IPancakePairCalls::Decimals(element) => element.fmt(f),
                IPancakePairCalls::Factory(element) => element.fmt(f),
                IPancakePairCalls::GetReserves(element) => element.fmt(f),
                IPancakePairCalls::Initialize(element) => element.fmt(f),
                IPancakePairCalls::Klast(element) => element.fmt(f),
                IPancakePairCalls::Mint(element) => element.fmt(f),
                IPancakePairCalls::Name(element) => element.fmt(f),
                IPancakePairCalls::Nonces(element) => element.fmt(f),
                IPancakePairCalls::Permit(element) => element.fmt(f),
                IPancakePairCalls::Price0CumulativeLast(element) => element.fmt(f),
                IPancakePairCalls::Price1CumulativeLast(element) => element.fmt(f),
                IPancakePairCalls::Skim(element) => element.fmt(f),
                IPancakePairCalls::Swap(element) => element.fmt(f),
                IPancakePairCalls::Symbol(element) => element.fmt(f),
                IPancakePairCalls::Sync(element) => element.fmt(f),
                IPancakePairCalls::Token0(element) => element.fmt(f),
                IPancakePairCalls::Token1(element) => element.fmt(f),
                IPancakePairCalls::TotalSupply(element) => element.fmt(f),
                IPancakePairCalls::Transfer(element) => element.fmt(f),
                IPancakePairCalls::TransferFrom(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<DomainSeparatorCall> for IPancakePairCalls {
        fn from(var: DomainSeparatorCall) -> Self {
            IPancakePairCalls::DomainSeparator(var)
        }
    }
    impl ::std::convert::From<MinimumLiquidityCall> for IPancakePairCalls {
        fn from(var: MinimumLiquidityCall) -> Self {
            IPancakePairCalls::MinimumLiquidity(var)
        }
    }
    impl ::std::convert::From<PermitTypehashCall> for IPancakePairCalls {
        fn from(var: PermitTypehashCall) -> Self {
            IPancakePairCalls::PermitTypehash(var)
        }
    }
    impl ::std::convert::From<AllowanceCall> for IPancakePairCalls {
        fn from(var: AllowanceCall) -> Self {
            IPancakePairCalls::Allowance(var)
        }
    }
    impl ::std::convert::From<ApproveCall> for IPancakePairCalls {
        fn from(var: ApproveCall) -> Self {
            IPancakePairCalls::Approve(var)
        }
    }
    impl ::std::convert::From<BalanceOfCall> for IPancakePairCalls {
        fn from(var: BalanceOfCall) -> Self {
            IPancakePairCalls::BalanceOf(var)
        }
    }
    impl ::std::convert::From<BurnCall> for IPancakePairCalls {
        fn from(var: BurnCall) -> Self {
            IPancakePairCalls::Burn(var)
        }
    }
    impl ::std::convert::From<DecimalsCall> for IPancakePairCalls {
        fn from(var: DecimalsCall) -> Self {
            IPancakePairCalls::Decimals(var)
        }
    }
    impl ::std::convert::From<FactoryCall> for IPancakePairCalls {
        fn from(var: FactoryCall) -> Self {
            IPancakePairCalls::Factory(var)
        }
    }
    impl ::std::convert::From<GetReservesCall> for IPancakePairCalls {
        fn from(var: GetReservesCall) -> Self {
            IPancakePairCalls::GetReserves(var)
        }
    }
    impl ::std::convert::From<InitializeCall> for IPancakePairCalls {
        fn from(var: InitializeCall) -> Self {
            IPancakePairCalls::Initialize(var)
        }
    }
    impl ::std::convert::From<KlastCall> for IPancakePairCalls {
        fn from(var: KlastCall) -> Self {
            IPancakePairCalls::Klast(var)
        }
    }
    impl ::std::convert::From<MintCall> for IPancakePairCalls {
        fn from(var: MintCall) -> Self {
            IPancakePairCalls::Mint(var)
        }
    }
    impl ::std::convert::From<NameCall> for IPancakePairCalls {
        fn from(var: NameCall) -> Self {
            IPancakePairCalls::Name(var)
        }
    }
    impl ::std::convert::From<NoncesCall> for IPancakePairCalls {
        fn from(var: NoncesCall) -> Self {
            IPancakePairCalls::Nonces(var)
        }
    }
    impl ::std::convert::From<PermitCall> for IPancakePairCalls {
        fn from(var: PermitCall) -> Self {
            IPancakePairCalls::Permit(var)
        }
    }
    impl ::std::convert::From<Price0CumulativeLastCall> for IPancakePairCalls {
        fn from(var: Price0CumulativeLastCall) -> Self {
            IPancakePairCalls::Price0CumulativeLast(var)
        }
    }
    impl ::std::convert::From<Price1CumulativeLastCall> for IPancakePairCalls {
        fn from(var: Price1CumulativeLastCall) -> Self {
            IPancakePairCalls::Price1CumulativeLast(var)
        }
    }
    impl ::std::convert::From<SkimCall> for IPancakePairCalls {
        fn from(var: SkimCall) -> Self {
            IPancakePairCalls::Skim(var)
        }
    }
    impl ::std::convert::From<SwapCall> for IPancakePairCalls {
        fn from(var: SwapCall) -> Self {
            IPancakePairCalls::Swap(var)
        }
    }
    impl ::std::convert::From<SymbolCall> for IPancakePairCalls {
        fn from(var: SymbolCall) -> Self {
            IPancakePairCalls::Symbol(var)
        }
    }
    impl ::std::convert::From<SyncCall> for IPancakePairCalls {
        fn from(var: SyncCall) -> Self {
            IPancakePairCalls::Sync(var)
        }
    }
    impl ::std::convert::From<Token0Call> for IPancakePairCalls {
        fn from(var: Token0Call) -> Self {
            IPancakePairCalls::Token0(var)
        }
    }
    impl ::std::convert::From<Token1Call> for IPancakePairCalls {
        fn from(var: Token1Call) -> Self {
            IPancakePairCalls::Token1(var)
        }
    }
    impl ::std::convert::From<TotalSupplyCall> for IPancakePairCalls {
        fn from(var: TotalSupplyCall) -> Self {
            IPancakePairCalls::TotalSupply(var)
        }
    }
    impl ::std::convert::From<TransferCall> for IPancakePairCalls {
        fn from(var: TransferCall) -> Self {
            IPancakePairCalls::Transfer(var)
        }
    }
    impl ::std::convert::From<TransferFromCall> for IPancakePairCalls {
        fn from(var: TransferFromCall) -> Self {
            IPancakePairCalls::TransferFrom(var)
        }
    }
    #[doc = "Container type for all return fields from the `DOMAIN_SEPARATOR` function with signature `DOMAIN_SEPARATOR()` and selector `[54, 68, 229, 21]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct DomainSeparatorReturn(pub [u8; 32]);
    #[doc = "Container type for all return fields from the `MINIMUM_LIQUIDITY` function with signature `MINIMUM_LIQUIDITY()` and selector `[186, 154, 122, 86]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct MinimumLiquidityReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `PERMIT_TYPEHASH` function with signature `PERMIT_TYPEHASH()` and selector `[48, 173, 248, 31]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct PermitTypehashReturn(pub [u8; 32]);
    #[doc = "Container type for all return fields from the `allowance` function with signature `allowance(address,address)` and selector `[221, 98, 237, 62]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct AllowanceReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `approve` function with signature `approve(address,uint256)` and selector `[9, 94, 167, 179]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ApproveReturn(pub bool);
    #[doc = "Container type for all return fields from the `balanceOf` function with signature `balanceOf(address)` and selector `[112, 160, 130, 49]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct BalanceOfReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `burn` function with signature `burn(address)` and selector `[137, 175, 203, 68]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct BurnReturn {
        pub amount_0: ethers::core::types::U256,
        pub amount_1: ethers::core::types::U256,
    }
    #[doc = "Container type for all return fields from the `decimals` function with signature `decimals()` and selector `[49, 60, 229, 103]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct DecimalsReturn(pub u8);
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
    #[doc = "Container type for all return fields from the `getReserves` function with signature `getReserves()` and selector `[9, 2, 241, 172]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct GetReservesReturn {
        pub reserve_0: u128,
        pub reserve_1: u128,
        pub block_timestamp_last: u32,
    }
    #[doc = "Container type for all return fields from the `kLast` function with signature `kLast()` and selector `[116, 100, 252, 61]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct KlastReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `mint` function with signature `mint(address)` and selector `[106, 98, 120, 66]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct MintReturn {
        pub liquidity: ethers::core::types::U256,
    }
    #[doc = "Container type for all return fields from the `name` function with signature `name()` and selector `[6, 253, 222, 3]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct NameReturn(pub String);
    #[doc = "Container type for all return fields from the `nonces` function with signature `nonces(address)` and selector `[126, 206, 190, 0]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct NoncesReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `price0CumulativeLast` function with signature `price0CumulativeLast()` and selector `[89, 9, 192, 213]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct Price0CumulativeLastReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `price1CumulativeLast` function with signature `price1CumulativeLast()` and selector `[90, 61, 84, 147]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct Price1CumulativeLastReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `symbol` function with signature `symbol()` and selector `[149, 216, 155, 65]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct SymbolReturn(pub String);
    #[doc = "Container type for all return fields from the `token0` function with signature `token0()` and selector `[13, 254, 22, 129]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct Token0Return(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `token1` function with signature `token1()` and selector `[210, 18, 32, 167]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct Token1Return(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `totalSupply` function with signature `totalSupply()` and selector `[24, 22, 13, 221]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct TotalSupplyReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `transfer` function with signature `transfer(address,uint256)` and selector `[169, 5, 156, 187]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct TransferReturn(pub bool);
    #[doc = "Container type for all return fields from the `transferFrom` function with signature `transferFrom(address,address,uint256)` and selector `[35, 184, 114, 221]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct TransferFromReturn(pub bool);
}
