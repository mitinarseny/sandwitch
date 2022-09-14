pub use pancake_factory_v2::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod pancake_factory_v2 {
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
    #[doc = "PancakeFactoryV2 was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    pub static PANCAKEFACTORYV2_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers :: core :: utils :: __serde_json :: from_str ("[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_feeToSetter\",\"type\":\"address\"}],\"payable\":false,\"stateMutability\":\"nonpayable\",\"type\":\"constructor\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"address\",\"name\":\"token0\",\"type\":\"address\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"token1\",\"type\":\"address\"},{\"indexed\":false,\"internalType\":\"address\",\"name\":\"pair\",\"type\":\"address\"},{\"indexed\":false,\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"name\":\"PairCreated\",\"type\":\"event\"},{\"constant\":true,\"inputs\":[],\"name\":\"INIT_CODE_PAIR_HASH\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"payable\":false,\"stateMutability\":\"view\",\"type\":\"function\"},{\"constant\":true,\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"name\":\"allPairs\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\"}],\"payable\":false,\"stateMutability\":\"view\",\"type\":\"function\"},{\"constant\":true,\"inputs\":[],\"name\":\"allPairsLength\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"payable\":false,\"stateMutability\":\"view\",\"type\":\"function\"},{\"constant\":false,\"inputs\":[{\"internalType\":\"address\",\"name\":\"tokenA\",\"type\":\"address\"},{\"internalType\":\"address\",\"name\":\"tokenB\",\"type\":\"address\"}],\"name\":\"createPair\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"pair\",\"type\":\"address\"}],\"payable\":false,\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"constant\":true,\"inputs\":[],\"name\":\"feeTo\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\"}],\"payable\":false,\"stateMutability\":\"view\",\"type\":\"function\"},{\"constant\":true,\"inputs\":[],\"name\":\"feeToSetter\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\"}],\"payable\":false,\"stateMutability\":\"view\",\"type\":\"function\"},{\"constant\":true,\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\"},{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\"}],\"name\":\"getPair\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\"}],\"payable\":false,\"stateMutability\":\"view\",\"type\":\"function\"},{\"constant\":false,\"inputs\":[{\"internalType\":\"address\",\"name\":\"_feeTo\",\"type\":\"address\"}],\"name\":\"setFeeTo\",\"outputs\":[],\"payable\":false,\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"constant\":false,\"inputs\":[{\"internalType\":\"address\",\"name\":\"_feeToSetter\",\"type\":\"address\"}],\"name\":\"setFeeToSetter\",\"outputs\":[],\"payable\":false,\"stateMutability\":\"nonpayable\",\"type\":\"function\"}]\n") . expect ("invalid abi")
        });
    pub struct PancakeFactoryV2<M>(ethers::contract::Contract<M>);
    impl<M> Clone for PancakeFactoryV2<M> {
        fn clone(&self) -> Self {
            PancakeFactoryV2(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for PancakeFactoryV2<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M: ethers::providers::Middleware> std::fmt::Debug for PancakeFactoryV2<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(PancakeFactoryV2))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> PancakeFactoryV2<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), PANCAKEFACTORYV2_ABI.clone(), client)
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
            p0: ethers::core::types::Address,
            p1: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([230, 164, 57, 5], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `setFeeTo` (0xf46901ed) function"]
        pub fn set_fee_to(
            &self,
            fee_to: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([244, 105, 1, 237], fee_to)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `setFeeToSetter` (0xa2e74af6) function"]
        pub fn set_fee_to_setter(
            &self,
            fee_to_setter: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([162, 231, 74, 246], fee_to_setter)
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
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for PancakeFactoryV2<M> {
        fn from(contract: ethers::contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
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
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
    )]
    #[ethcall(name = "INIT_CODE_PAIR_HASH", abi = "INIT_CODE_PAIR_HASH()")]
    pub struct InitCodePairHashCall;
    #[doc = "Container type for all input parameters for the `allPairs` function with signature `allPairs(uint256)` and selector `[30, 61, 209, 139]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
    )]
    #[ethcall(name = "allPairs", abi = "allPairs(uint256)")]
    pub struct AllPairsCall(pub ethers::core::types::U256);
    #[doc = "Container type for all input parameters for the `allPairsLength` function with signature `allPairsLength()` and selector `[87, 79, 43, 163]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
    )]
    #[ethcall(name = "allPairsLength", abi = "allPairsLength()")]
    pub struct AllPairsLengthCall;
    #[doc = "Container type for all input parameters for the `createPair` function with signature `createPair(address,address)` and selector `[201, 198, 83, 150]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
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
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
    )]
    #[ethcall(name = "feeTo", abi = "feeTo()")]
    pub struct FeeToCall;
    #[doc = "Container type for all input parameters for the `feeToSetter` function with signature `feeToSetter()` and selector `[9, 75, 116, 21]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
    )]
    #[ethcall(name = "feeToSetter", abi = "feeToSetter()")]
    pub struct FeeToSetterCall;
    #[doc = "Container type for all input parameters for the `getPair` function with signature `getPair(address,address)` and selector `[230, 164, 57, 5]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
    )]
    #[ethcall(name = "getPair", abi = "getPair(address,address)")]
    pub struct GetPairCall(
        pub ethers::core::types::Address,
        pub ethers::core::types::Address,
    );
    #[doc = "Container type for all input parameters for the `setFeeTo` function with signature `setFeeTo(address)` and selector `[244, 105, 1, 237]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
    )]
    #[ethcall(name = "setFeeTo", abi = "setFeeTo(address)")]
    pub struct SetFeeToCall {
        pub fee_to: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `setFeeToSetter` function with signature `setFeeToSetter(address)` and selector `[162, 231, 74, 246]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
    )]
    #[ethcall(name = "setFeeToSetter", abi = "setFeeToSetter(address)")]
    pub struct SetFeeToSetterCall {
        pub fee_to_setter: ethers::core::types::Address,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum PancakeFactoryV2Calls {
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
    impl ethers::core::abi::AbiDecode for PancakeFactoryV2Calls {
        fn decode(data: impl AsRef<[u8]>) -> Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <InitCodePairHashCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeFactoryV2Calls::InitCodePairHash(decoded));
            }
            if let Ok(decoded) =
                <AllPairsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeFactoryV2Calls::AllPairs(decoded));
            }
            if let Ok(decoded) =
                <AllPairsLengthCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeFactoryV2Calls::AllPairsLength(decoded));
            }
            if let Ok(decoded) =
                <CreatePairCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeFactoryV2Calls::CreatePair(decoded));
            }
            if let Ok(decoded) = <FeeToCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeFactoryV2Calls::FeeTo(decoded));
            }
            if let Ok(decoded) =
                <FeeToSetterCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeFactoryV2Calls::FeeToSetter(decoded));
            }
            if let Ok(decoded) =
                <GetPairCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeFactoryV2Calls::GetPair(decoded));
            }
            if let Ok(decoded) =
                <SetFeeToCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeFactoryV2Calls::SetFeeTo(decoded));
            }
            if let Ok(decoded) =
                <SetFeeToSetterCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeFactoryV2Calls::SetFeeToSetter(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for PancakeFactoryV2Calls {
        fn encode(self) -> Vec<u8> {
            match self {
                PancakeFactoryV2Calls::InitCodePairHash(element) => element.encode(),
                PancakeFactoryV2Calls::AllPairs(element) => element.encode(),
                PancakeFactoryV2Calls::AllPairsLength(element) => element.encode(),
                PancakeFactoryV2Calls::CreatePair(element) => element.encode(),
                PancakeFactoryV2Calls::FeeTo(element) => element.encode(),
                PancakeFactoryV2Calls::FeeToSetter(element) => element.encode(),
                PancakeFactoryV2Calls::GetPair(element) => element.encode(),
                PancakeFactoryV2Calls::SetFeeTo(element) => element.encode(),
                PancakeFactoryV2Calls::SetFeeToSetter(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for PancakeFactoryV2Calls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                PancakeFactoryV2Calls::InitCodePairHash(element) => element.fmt(f),
                PancakeFactoryV2Calls::AllPairs(element) => element.fmt(f),
                PancakeFactoryV2Calls::AllPairsLength(element) => element.fmt(f),
                PancakeFactoryV2Calls::CreatePair(element) => element.fmt(f),
                PancakeFactoryV2Calls::FeeTo(element) => element.fmt(f),
                PancakeFactoryV2Calls::FeeToSetter(element) => element.fmt(f),
                PancakeFactoryV2Calls::GetPair(element) => element.fmt(f),
                PancakeFactoryV2Calls::SetFeeTo(element) => element.fmt(f),
                PancakeFactoryV2Calls::SetFeeToSetter(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<InitCodePairHashCall> for PancakeFactoryV2Calls {
        fn from(var: InitCodePairHashCall) -> Self {
            PancakeFactoryV2Calls::InitCodePairHash(var)
        }
    }
    impl ::std::convert::From<AllPairsCall> for PancakeFactoryV2Calls {
        fn from(var: AllPairsCall) -> Self {
            PancakeFactoryV2Calls::AllPairs(var)
        }
    }
    impl ::std::convert::From<AllPairsLengthCall> for PancakeFactoryV2Calls {
        fn from(var: AllPairsLengthCall) -> Self {
            PancakeFactoryV2Calls::AllPairsLength(var)
        }
    }
    impl ::std::convert::From<CreatePairCall> for PancakeFactoryV2Calls {
        fn from(var: CreatePairCall) -> Self {
            PancakeFactoryV2Calls::CreatePair(var)
        }
    }
    impl ::std::convert::From<FeeToCall> for PancakeFactoryV2Calls {
        fn from(var: FeeToCall) -> Self {
            PancakeFactoryV2Calls::FeeTo(var)
        }
    }
    impl ::std::convert::From<FeeToSetterCall> for PancakeFactoryV2Calls {
        fn from(var: FeeToSetterCall) -> Self {
            PancakeFactoryV2Calls::FeeToSetter(var)
        }
    }
    impl ::std::convert::From<GetPairCall> for PancakeFactoryV2Calls {
        fn from(var: GetPairCall) -> Self {
            PancakeFactoryV2Calls::GetPair(var)
        }
    }
    impl ::std::convert::From<SetFeeToCall> for PancakeFactoryV2Calls {
        fn from(var: SetFeeToCall) -> Self {
            PancakeFactoryV2Calls::SetFeeTo(var)
        }
    }
    impl ::std::convert::From<SetFeeToSetterCall> for PancakeFactoryV2Calls {
        fn from(var: SetFeeToSetterCall) -> Self {
            PancakeFactoryV2Calls::SetFeeToSetter(var)
        }
    }
    #[doc = "Container type for all return fields from the `INIT_CODE_PAIR_HASH` function with signature `INIT_CODE_PAIR_HASH()` and selector `[88, 85, 162, 90]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
    )]
    pub struct InitCodePairHashReturn(pub [u8; 32]);
    #[doc = "Container type for all return fields from the `allPairs` function with signature `allPairs(uint256)` and selector `[30, 61, 209, 139]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
    )]
    pub struct AllPairsReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `allPairsLength` function with signature `allPairsLength()` and selector `[87, 79, 43, 163]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
    )]
    pub struct AllPairsLengthReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `createPair` function with signature `createPair(address,address)` and selector `[201, 198, 83, 150]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
    )]
    pub struct CreatePairReturn {
        pub pair: ethers::core::types::Address,
    }
    #[doc = "Container type for all return fields from the `feeTo` function with signature `feeTo()` and selector `[1, 126, 126, 88]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
    )]
    pub struct FeeToReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `feeToSetter` function with signature `feeToSetter()` and selector `[9, 75, 116, 21]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
    )]
    pub struct FeeToSetterReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `getPair` function with signature `getPair(address,address)` and selector `[230, 164, 57, 5]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
    )]
    pub struct GetPairReturn(pub ethers::core::types::Address);
}
