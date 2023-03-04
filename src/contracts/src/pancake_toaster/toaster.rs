pub use toaster::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod toaster {
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
    #[doc = "Toaster was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"type\":\"error\",\"name\":\"InsufficientBalance\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"contract IERC20\",\"name\":\"token\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"type\":\"error\",\"name\":\"InsufficientTokenBalance\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"Uncled\",\"outputs\":[]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static TOASTER_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    pub struct Toaster<M>(ethers::contract::Contract<M>);
    impl<M> Clone for Toaster<M> {
        fn clone(&self) -> Self {
            Toaster(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for Toaster<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for Toaster<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(Toaster))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> Toaster<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), TOASTER_ABI.clone(), client).into()
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for Toaster<M> {
        fn from(contract: ethers::contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
    #[doc = "Custom Error type `InsufficientBalance` with signature `InsufficientBalance(address)` and selector `[137, 127, 108, 88]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthError,
        ethers :: contract :: EthDisplay,
    )]
    #[etherror(name = "InsufficientBalance", abi = "InsufficientBalance(address)")]
    pub struct InsufficientBalance {
        pub account: ethers::core::types::Address,
    }
    #[doc = "Custom Error type `InsufficientTokenBalance` with signature `InsufficientTokenBalance(address,address)` and selector `[186, 22, 5, 250]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthError,
        ethers :: contract :: EthDisplay,
    )]
    #[etherror(
        name = "InsufficientTokenBalance",
        abi = "InsufficientTokenBalance(address,address)"
    )]
    pub struct InsufficientTokenBalance {
        pub token: ethers::core::types::Address,
        pub account: ethers::core::types::Address,
    }
    #[doc = "Custom Error type `Uncled` with signature `Uncled()` and selector `[119, 151, 174, 109]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthError,
        ethers :: contract :: EthDisplay,
    )]
    #[etherror(name = "Uncled", abi = "Uncled()")]
    pub struct Uncled;
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum ToasterErrors {
        InsufficientBalance(InsufficientBalance),
        InsufficientTokenBalance(InsufficientTokenBalance),
        Uncled(Uncled),
    }
    impl ethers::core::abi::AbiDecode for ToasterErrors {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <InsufficientBalance as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ToasterErrors::InsufficientBalance(decoded));
            }
            if let Ok(decoded) =
                <InsufficientTokenBalance as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ToasterErrors::InsufficientTokenBalance(decoded));
            }
            if let Ok(decoded) = <Uncled as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(ToasterErrors::Uncled(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for ToasterErrors {
        fn encode(self) -> Vec<u8> {
            match self {
                ToasterErrors::InsufficientBalance(element) => element.encode(),
                ToasterErrors::InsufficientTokenBalance(element) => element.encode(),
                ToasterErrors::Uncled(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for ToasterErrors {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                ToasterErrors::InsufficientBalance(element) => element.fmt(f),
                ToasterErrors::InsufficientTokenBalance(element) => element.fmt(f),
                ToasterErrors::Uncled(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<InsufficientBalance> for ToasterErrors {
        fn from(var: InsufficientBalance) -> Self {
            ToasterErrors::InsufficientBalance(var)
        }
    }
    impl ::std::convert::From<InsufficientTokenBalance> for ToasterErrors {
        fn from(var: InsufficientTokenBalance) -> Self {
            ToasterErrors::InsufficientTokenBalance(var)
        }
    }
    impl ::std::convert::From<Uncled> for ToasterErrors {
        fn from(var: Uncled) -> Self {
            ToasterErrors::Uncled(var)
        }
    }
}
