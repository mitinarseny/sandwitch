pub use pancake_library::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod pancake_library {
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
    #[doc = "PancakeLibrary was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[],\"type\":\"error\",\"name\":\"InsufficientInputAmount\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InsufficientLiquidity\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InsufficientOutputAmount\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InvalidPath\",\"outputs\":[]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static PANCAKELIBRARY_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    #[doc = r" Bytecode of the #name contract"]
    pub static PANCAKELIBRARY_BYTECODE: ethers::contract::Lazy<ethers::core::types::Bytes> =
        ethers::contract::Lazy::new(|| {
            "0x60566050600b82828239805160001a6073146043577f4e487b7100000000000000000000000000000000000000000000000000000000600052600060045260246000fd5b30600052607381538281f3fe73000000000000000000000000000000000000000030146080604052600080fdfea26469706673582212202cb0c8883c148c7ec2a45a1ca995c84dcaf86240d008f95d06a10abbc9be9f1964736f6c63430008130033" . parse () . expect ("invalid bytecode")
        });
    pub struct PancakeLibrary<M>(ethers::contract::Contract<M>);
    impl<M> Clone for PancakeLibrary<M> {
        fn clone(&self) -> Self {
            PancakeLibrary(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for PancakeLibrary<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for PancakeLibrary<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(PancakeLibrary))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> PancakeLibrary<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), PANCAKELIBRARY_ABI.clone(), client)
                .into()
        }
        #[doc = r" Constructs the general purpose `Deployer` instance based on the provided constructor arguments and sends it."]
        #[doc = r" Returns a new instance of a deployer that returns an instance of this contract after sending the transaction"]
        #[doc = r""]
        #[doc = r" Notes:"]
        #[doc = r" 1. If there are no constructor arguments, you should pass `()` as the argument."]
        #[doc = r" 1. The default poll duration is 7 seconds."]
        #[doc = r" 1. The default number of confirmations is 1 block."]
        #[doc = r""]
        #[doc = r""]
        #[doc = r" # Example"]
        #[doc = r""]
        #[doc = r" Generate contract bindings with `abigen!` and deploy a new contract instance."]
        #[doc = r""]
        #[doc = r" *Note*: this requires a `bytecode` and `abi` object in the `greeter.json` artifact."]
        #[doc = r""]
        #[doc = r" ```ignore"]
        #[doc = r" # async fn deploy<M: ethers::providers::Middleware>(client: ::std::sync::Arc<M>) {"]
        #[doc = r#"     abigen!(Greeter,"../greeter.json");"#]
        #[doc = r""]
        #[doc = r#"    let greeter_contract = Greeter::deploy(client, "Hello world!".to_string()).unwrap().send().await.unwrap();"#]
        #[doc = r"    let msg = greeter_contract.greet().call().await.unwrap();"]
        #[doc = r" # }"]
        #[doc = r" ```"]
        pub fn deploy<T: ethers::core::abi::Tokenize>(
            client: ::std::sync::Arc<M>,
            constructor_args: T,
        ) -> ::std::result::Result<
            ethers::contract::builders::ContractDeployer<M, Self>,
            ethers::contract::ContractError<M>,
        > {
            let factory = ethers::contract::ContractFactory::new(
                PANCAKELIBRARY_ABI.clone(),
                PANCAKELIBRARY_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for PancakeLibrary<M> {
        fn from(contract: ethers::contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
    #[doc = "Custom Error type `InsufficientInputAmount` with signature `InsufficientInputAmount()` and selector `[9, 143, 181, 97]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthError,
        ethers :: contract :: EthDisplay,
    )]
    #[etherror(name = "InsufficientInputAmount", abi = "InsufficientInputAmount()")]
    pub struct InsufficientInputAmount;
    #[doc = "Custom Error type `InsufficientLiquidity` with signature `InsufficientLiquidity()` and selector `[187, 85, 253, 39]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthError,
        ethers :: contract :: EthDisplay,
    )]
    #[etherror(name = "InsufficientLiquidity", abi = "InsufficientLiquidity()")]
    pub struct InsufficientLiquidity;
    #[doc = "Custom Error type `InsufficientOutputAmount` with signature `InsufficientOutputAmount()` and selector `[66, 48, 28, 35]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthError,
        ethers :: contract :: EthDisplay,
    )]
    #[etherror(name = "InsufficientOutputAmount", abi = "InsufficientOutputAmount()")]
    pub struct InsufficientOutputAmount;
    #[doc = "Custom Error type `InvalidPath` with signature `InvalidPath()` and selector `[32, 219, 130, 103]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthError,
        ethers :: contract :: EthDisplay,
    )]
    #[etherror(name = "InvalidPath", abi = "InvalidPath()")]
    pub struct InvalidPath;
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum PancakeLibraryErrors {
        InsufficientInputAmount(InsufficientInputAmount),
        InsufficientLiquidity(InsufficientLiquidity),
        InsufficientOutputAmount(InsufficientOutputAmount),
        InvalidPath(InvalidPath),
    }
    impl ethers::core::abi::AbiDecode for PancakeLibraryErrors {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <InsufficientInputAmount as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeLibraryErrors::InsufficientInputAmount(decoded));
            }
            if let Ok(decoded) =
                <InsufficientLiquidity as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeLibraryErrors::InsufficientLiquidity(decoded));
            }
            if let Ok(decoded) =
                <InsufficientOutputAmount as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeLibraryErrors::InsufficientOutputAmount(decoded));
            }
            if let Ok(decoded) =
                <InvalidPath as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeLibraryErrors::InvalidPath(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for PancakeLibraryErrors {
        fn encode(self) -> Vec<u8> {
            match self {
                PancakeLibraryErrors::InsufficientInputAmount(element) => element.encode(),
                PancakeLibraryErrors::InsufficientLiquidity(element) => element.encode(),
                PancakeLibraryErrors::InsufficientOutputAmount(element) => element.encode(),
                PancakeLibraryErrors::InvalidPath(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for PancakeLibraryErrors {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                PancakeLibraryErrors::InsufficientInputAmount(element) => element.fmt(f),
                PancakeLibraryErrors::InsufficientLiquidity(element) => element.fmt(f),
                PancakeLibraryErrors::InsufficientOutputAmount(element) => element.fmt(f),
                PancakeLibraryErrors::InvalidPath(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<InsufficientInputAmount> for PancakeLibraryErrors {
        fn from(var: InsufficientInputAmount) -> Self {
            PancakeLibraryErrors::InsufficientInputAmount(var)
        }
    }
    impl ::std::convert::From<InsufficientLiquidity> for PancakeLibraryErrors {
        fn from(var: InsufficientLiquidity) -> Self {
            PancakeLibraryErrors::InsufficientLiquidity(var)
        }
    }
    impl ::std::convert::From<InsufficientOutputAmount> for PancakeLibraryErrors {
        fn from(var: InsufficientOutputAmount) -> Self {
            PancakeLibraryErrors::InsufficientOutputAmount(var)
        }
    }
    impl ::std::convert::From<InvalidPath> for PancakeLibraryErrors {
        fn from(var: InvalidPath) -> Self {
            PancakeLibraryErrors::InvalidPath(var)
        }
    }
}
