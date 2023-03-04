pub use i_pancake_factory::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod i_pancake_factory {
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
    #[doc = "IPancakeFactory was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"token0\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"token1\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"pair\",\"type\":\"address\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"PairCreated\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"INIT_CODE_PAIR_HASH\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"allPairs\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"pair\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"allPairsLength\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"tokenA\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"tokenB\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"createPair\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"pair\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"feeTo\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"feeToSetter\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"tokenA\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"tokenB\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getPair\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"pair\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"setFeeTo\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"setFeeToSetter\",\"outputs\":[]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static IPANCAKEFACTORY_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    pub struct IPancakeFactory<M>(ethers::contract::Contract<M>);
    impl<M> Clone for IPancakeFactory<M> {
        fn clone(&self) -> Self {
            IPancakeFactory(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for IPancakeFactory<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for IPancakeFactory<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(IPancakeFactory))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> IPancakeFactory<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), IPANCAKEFACTORY_ABI.clone(), client)
                .into()
        }
        #[doc = "Calls the contract's `INIT_CODE_PAIR_HASH` (0x5855a25a) function"]
        pub fn init_code_pair_hash(&self) -> ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([88, 85, 162, 90], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `allPairs` (0x1e3dd18b) function"]
        pub fn all_pairs(
            &self,
            p0: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([30, 61, 209, 139], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `allPairsLength` (0x574f2ba3) function"]
        pub fn all_pairs_length(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([87, 79, 43, 163], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `createPair` (0xc9c65396) function"]
        pub fn create_pair(
            &self,
            token_a: ethers::core::types::Address,
            token_b: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([201, 198, 83, 150], (token_a, token_b))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `feeTo` (0x017e7e58) function"]
        pub fn fee_to(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([1, 126, 126, 88], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `feeToSetter` (0x094b7415) function"]
        pub fn fee_to_setter(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([9, 75, 116, 21], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getPair` (0xe6a43905) function"]
        pub fn get_pair(
            &self,
            token_a: ethers::core::types::Address,
            token_b: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([230, 164, 57, 5], (token_a, token_b))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `setFeeTo` (0xf46901ed) function"]
        pub fn set_fee_to(
            &self,
            p0: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([244, 105, 1, 237], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `setFeeToSetter` (0xa2e74af6) function"]
        pub fn set_fee_to_setter(
            &self,
            p0: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([162, 231, 74, 246], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Gets the contract's `PairCreated` event"]
        pub fn pair_created_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, PairCreatedFilter> {
            self.0.event()
        }
        #[doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(&self) -> ethers::contract::builders::Event<M, PairCreatedFilter> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for IPancakeFactory<M> {
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
    #[ethevent(
        name = "PairCreated",
        abi = "PairCreated(address,address,address,uint256)"
    )]
    pub struct PairCreatedFilter {
        #[ethevent(indexed)]
        pub token_0: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub token_1: ethers::core::types::Address,
        pub pair: ethers::core::types::Address,
        pub p3: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `INIT_CODE_PAIR_HASH` function with signature `INIT_CODE_PAIR_HASH()` and selector `[88, 85, 162, 90]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "INIT_CODE_PAIR_HASH", abi = "INIT_CODE_PAIR_HASH()")]
    pub struct InitCodePairHashCall;
    #[doc = "Container type for all input parameters for the `allPairs` function with signature `allPairs(uint256)` and selector `[30, 61, 209, 139]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "allPairs", abi = "allPairs(uint256)")]
    pub struct AllPairsCall(pub ethers::core::types::U256);
    #[doc = "Container type for all input parameters for the `allPairsLength` function with signature `allPairsLength()` and selector `[87, 79, 43, 163]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "allPairsLength", abi = "allPairsLength()")]
    pub struct AllPairsLengthCall;
    #[doc = "Container type for all input parameters for the `createPair` function with signature `createPair(address,address)` and selector `[201, 198, 83, 150]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "createPair", abi = "createPair(address,address)")]
    pub struct CreatePairCall {
        pub token_a: ethers::core::types::Address,
        pub token_b: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `feeTo` function with signature `feeTo()` and selector `[1, 126, 126, 88]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "feeTo", abi = "feeTo()")]
    pub struct FeeToCall;
    #[doc = "Container type for all input parameters for the `feeToSetter` function with signature `feeToSetter()` and selector `[9, 75, 116, 21]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "feeToSetter", abi = "feeToSetter()")]
    pub struct FeeToSetterCall;
    #[doc = "Container type for all input parameters for the `getPair` function with signature `getPair(address,address)` and selector `[230, 164, 57, 5]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "getPair", abi = "getPair(address,address)")]
    pub struct GetPairCall {
        pub token_a: ethers::core::types::Address,
        pub token_b: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `setFeeTo` function with signature `setFeeTo(address)` and selector `[244, 105, 1, 237]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "setFeeTo", abi = "setFeeTo(address)")]
    pub struct SetFeeToCall(pub ethers::core::types::Address);
    #[doc = "Container type for all input parameters for the `setFeeToSetter` function with signature `setFeeToSetter(address)` and selector `[162, 231, 74, 246]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "setFeeToSetter", abi = "setFeeToSetter(address)")]
    pub struct SetFeeToSetterCall(pub ethers::core::types::Address);
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum IPancakeFactoryCalls {
        InitCodePairHash(InitCodePairHashCall),
        AllPairs(AllPairsCall),
        AllPairsLength(AllPairsLengthCall),
        CreatePair(CreatePairCall),
        FeeTo(FeeToCall),
        FeeToSetter(FeeToSetterCall),
        GetPair(GetPairCall),
        SetFeeTo(SetFeeToCall),
        SetFeeToSetter(SetFeeToSetterCall),
    }
    impl ethers::core::abi::AbiDecode for IPancakeFactoryCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <InitCodePairHashCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakeFactoryCalls::InitCodePairHash(decoded));
            }
            if let Ok(decoded) =
                <AllPairsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakeFactoryCalls::AllPairs(decoded));
            }
            if let Ok(decoded) =
                <AllPairsLengthCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakeFactoryCalls::AllPairsLength(decoded));
            }
            if let Ok(decoded) =
                <CreatePairCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakeFactoryCalls::CreatePair(decoded));
            }
            if let Ok(decoded) = <FeeToCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakeFactoryCalls::FeeTo(decoded));
            }
            if let Ok(decoded) =
                <FeeToSetterCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakeFactoryCalls::FeeToSetter(decoded));
            }
            if let Ok(decoded) =
                <GetPairCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakeFactoryCalls::GetPair(decoded));
            }
            if let Ok(decoded) =
                <SetFeeToCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakeFactoryCalls::SetFeeTo(decoded));
            }
            if let Ok(decoded) =
                <SetFeeToSetterCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IPancakeFactoryCalls::SetFeeToSetter(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for IPancakeFactoryCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                IPancakeFactoryCalls::InitCodePairHash(element) => element.encode(),
                IPancakeFactoryCalls::AllPairs(element) => element.encode(),
                IPancakeFactoryCalls::AllPairsLength(element) => element.encode(),
                IPancakeFactoryCalls::CreatePair(element) => element.encode(),
                IPancakeFactoryCalls::FeeTo(element) => element.encode(),
                IPancakeFactoryCalls::FeeToSetter(element) => element.encode(),
                IPancakeFactoryCalls::GetPair(element) => element.encode(),
                IPancakeFactoryCalls::SetFeeTo(element) => element.encode(),
                IPancakeFactoryCalls::SetFeeToSetter(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for IPancakeFactoryCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                IPancakeFactoryCalls::InitCodePairHash(element) => element.fmt(f),
                IPancakeFactoryCalls::AllPairs(element) => element.fmt(f),
                IPancakeFactoryCalls::AllPairsLength(element) => element.fmt(f),
                IPancakeFactoryCalls::CreatePair(element) => element.fmt(f),
                IPancakeFactoryCalls::FeeTo(element) => element.fmt(f),
                IPancakeFactoryCalls::FeeToSetter(element) => element.fmt(f),
                IPancakeFactoryCalls::GetPair(element) => element.fmt(f),
                IPancakeFactoryCalls::SetFeeTo(element) => element.fmt(f),
                IPancakeFactoryCalls::SetFeeToSetter(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<InitCodePairHashCall> for IPancakeFactoryCalls {
        fn from(var: InitCodePairHashCall) -> Self {
            IPancakeFactoryCalls::InitCodePairHash(var)
        }
    }
    impl ::std::convert::From<AllPairsCall> for IPancakeFactoryCalls {
        fn from(var: AllPairsCall) -> Self {
            IPancakeFactoryCalls::AllPairs(var)
        }
    }
    impl ::std::convert::From<AllPairsLengthCall> for IPancakeFactoryCalls {
        fn from(var: AllPairsLengthCall) -> Self {
            IPancakeFactoryCalls::AllPairsLength(var)
        }
    }
    impl ::std::convert::From<CreatePairCall> for IPancakeFactoryCalls {
        fn from(var: CreatePairCall) -> Self {
            IPancakeFactoryCalls::CreatePair(var)
        }
    }
    impl ::std::convert::From<FeeToCall> for IPancakeFactoryCalls {
        fn from(var: FeeToCall) -> Self {
            IPancakeFactoryCalls::FeeTo(var)
        }
    }
    impl ::std::convert::From<FeeToSetterCall> for IPancakeFactoryCalls {
        fn from(var: FeeToSetterCall) -> Self {
            IPancakeFactoryCalls::FeeToSetter(var)
        }
    }
    impl ::std::convert::From<GetPairCall> for IPancakeFactoryCalls {
        fn from(var: GetPairCall) -> Self {
            IPancakeFactoryCalls::GetPair(var)
        }
    }
    impl ::std::convert::From<SetFeeToCall> for IPancakeFactoryCalls {
        fn from(var: SetFeeToCall) -> Self {
            IPancakeFactoryCalls::SetFeeTo(var)
        }
    }
    impl ::std::convert::From<SetFeeToSetterCall> for IPancakeFactoryCalls {
        fn from(var: SetFeeToSetterCall) -> Self {
            IPancakeFactoryCalls::SetFeeToSetter(var)
        }
    }
    #[doc = "Container type for all return fields from the `INIT_CODE_PAIR_HASH` function with signature `INIT_CODE_PAIR_HASH()` and selector `[88, 85, 162, 90]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct InitCodePairHashReturn(pub [u8; 32]);
    #[doc = "Container type for all return fields from the `allPairs` function with signature `allPairs(uint256)` and selector `[30, 61, 209, 139]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct AllPairsReturn {
        pub pair: ethers::core::types::Address,
    }
    #[doc = "Container type for all return fields from the `allPairsLength` function with signature `allPairsLength()` and selector `[87, 79, 43, 163]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct AllPairsLengthReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `createPair` function with signature `createPair(address,address)` and selector `[201, 198, 83, 150]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct CreatePairReturn {
        pub pair: ethers::core::types::Address,
    }
    #[doc = "Container type for all return fields from the `feeTo` function with signature `feeTo()` and selector `[1, 126, 126, 88]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct FeeToReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `feeToSetter` function with signature `feeToSetter()` and selector `[9, 75, 116, 21]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct FeeToSetterReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `getPair` function with signature `getPair(address,address)` and selector `[230, 164, 57, 5]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct GetPairReturn {
        pub pair: ethers::core::types::Address,
    }
}
