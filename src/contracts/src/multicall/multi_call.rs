pub use multi_call::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod multi_call {
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
    #[doc = "MultiCall was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[],\"stateMutability\":\"payable\",\"type\":\"constructor\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"index\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]}],\"type\":\"error\",\"name\":\"Failed\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InvalidCommand\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"LengthMismatch\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"bytes\",\"name\":\"commands\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes[]\",\"name\":\"inputs\",\"type\":\"bytes[]\",\"components\":[]}],\"stateMutability\":\"payable\",\"type\":\"function\",\"name\":\"multicall\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"successes\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes[]\",\"name\":\"outputs\",\"type\":\"bytes[]\",\"components\":[]}]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static MULTICALL_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    #[doc = r" Bytecode of the #name contract"]
    pub static MULTICALL_BYTECODE: ethers::contract::Lazy<ethers::core::types::Bytes> =
        ethers::contract::Lazy::new(|| {
            "0x60806040526110c2806100136000396000f3fe60806040526004361061001e5760003560e01c80639171e82e14610023575b600080fd5b61003d60048036038101906100389190610954565b610054565b60405161004b929190610b71565b60405180910390f35b606080600086869050905080858590501461009b576040517fff633a3800000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b600060078216146100ad5760016100b0565b60005b60ff16600382901c6100c29190610be1565b67ffffffffffffffff8111156100db576100da610c15565b5b6040519080825280601f01601f19166020018201604052801561010d5781602001600182028036833780820191505090505b5092508067ffffffffffffffff81111561012a57610129610c15565b5b60405190808252806020026020018201604052801561015d57816020015b60608152602001906001900390816101485790505b509150600080606060005b84811015610311578a8a8281811061018357610182610c44565b5b9050013560f81c60f81b93508581815181106101a2576101a1610c44565b5b602002602001015191506102006007600160ff16901b60f81b19851660f81c60ff1660098111156101d6576101d5610c73565b5b8a8a848181106101e9576101e8610c44565b5b90506020028101906101fb9190610cb1565b61031f565b809350819450505082156102925760078116600160f81b7effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916901b87600383901c8151811061025257610251610c44565b5b6020010181815160f81c60f81b179150907effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916908160001a90535061030c565b600060f81b6007600160ff16901b60f81b85167effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff19160361030b5780826040517f173adf84000000000000000000000000000000000000000000000000000000008152600401610302929190610d23565b60405180910390fd5b5b610168565b505050505094509492505050565b600060606000600981111561033757610336610c73565b5b85600981111561034a57610349610c73565b5b036103e8573073ffffffffffffffffffffffffffffffffffffffff16639171e82e60e01b858560405160200161038293929190610ddf565b60405160208183030381529060405260405161039e9190610e3a565b600060405180830381855af49150503d80600081146103d9576040519150601f19603f3d011682016040523d82523d6000602084013e6103de565b606091505b5091509150610887565b600360098111156103fc576103fb610c73565b5b85600981111561040f5761040e610c73565b5b0361043f576001476040516020016104279190610e72565b60405160208183030381529060405291509150610887565b6004600981111561045357610452610c73565b5b85600981111561046657610465610c73565b5b036104ad5760013373ffffffffffffffffffffffffffffffffffffffff16316040516020016104959190610e72565b60405160208183030381529060405291509150610887565b600560098111156104c1576104c0610c73565b5b8560098111156104d4576104d3610c73565b5b036105305760008484906104e89190610ed1565b60601c905060018173ffffffffffffffffffffffffffffffffffffffff16316040516020016105179190610e72565b6040516020818303038152906040529250925050610887565b60006002600981111561054657610545610c73565b5b86600981111561055957610558610c73565b5b148061058957506007600981111561057457610573610c73565b5b86600981111561058757610586610c73565b5b145b806105b757506009808111156105a2576105a1610c73565b5b8660098111156105b5576105b4610c73565b5b145b156105e6578484906105c99190610f3a565b60001c9050848460209080926105e193929190610fa3565b945094505b600160098111156105fa576105f9610c73565b5b86600981111561060d5761060c610c73565b5b148061063d57506002600981111561062857610627610c73565b5b86600981111561063b5761063a610c73565b5b145b156106dd5760008585906106519190610ed1565b60601c90508073ffffffffffffffffffffffffffffffffffffffff16828787601490809261068193929190610fa3565b60405161068f929190610fde565b60006040518083038185875af1925050503d80600081146106cc576040519150601f19603f3d011682016040523d82523d6000602084013e6106d1565b606091505b50935093505050610887565b6000600660098111156106f3576106f2610c73565b5b87600981111561070657610705610c73565b5b148061073657506007600981111561072157610720610c73565b5b87600981111561073457610733610c73565b5b145b156107545760405185810160405285878237858184f0915050610828565b6008600981111561076857610767610c73565b5b87600981111561077b5761077a610c73565b5b14806107aa575060098081111561079557610794610c73565b5b8760098111156107a8576107a7610c73565b5b145b156107f55760008686906107be9190610f3a565b60001c9050868660209080926107d693929190610fa3565b965096506040518681016040528688823781878286f592505050610827565b6040517f12f269e500000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b5b600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614159350831561088457806040516020016108729190611071565b60405160208183030381529060405292505b50505b935093915050565b600080fd5b600080fd5b600080fd5b600080fd5b600080fd5b60008083601f8401126108be576108bd610899565b5b8235905067ffffffffffffffff8111156108db576108da61089e565b5b6020830191508360018202830111156108f7576108f66108a3565b5b9250929050565b60008083601f84011261091457610913610899565b5b8235905067ffffffffffffffff8111156109315761093061089e565b5b60208301915083602082028301111561094d5761094c6108a3565b5b9250929050565b6000806000806040858703121561096e5761096d61088f565b5b600085013567ffffffffffffffff81111561098c5761098b610894565b5b610998878288016108a8565b9450945050602085013567ffffffffffffffff8111156109bb576109ba610894565b5b6109c7878288016108fe565b925092505092959194509250565b600081519050919050565b600082825260208201905092915050565b60005b83811015610a0f5780820151818401526020810190506109f4565b60008484015250505050565b6000601f19601f8301169050919050565b6000610a37826109d5565b610a4181856109e0565b9350610a518185602086016109f1565b610a5a81610a1b565b840191505092915050565b600081519050919050565b600082825260208201905092915050565b6000819050602082019050919050565b600082825260208201905092915050565b6000610aad826109d5565b610ab78185610a91565b9350610ac78185602086016109f1565b610ad081610a1b565b840191505092915050565b6000610ae78383610aa2565b905092915050565b6000602082019050919050565b6000610b0782610a65565b610b118185610a70565b935083602082028501610b2385610a81565b8060005b85811015610b5f5784840389528151610b408582610adb565b9450610b4b83610aef565b925060208a01995050600181019050610b27565b50829750879550505050505092915050565b60006040820190508181036000830152610b8b8185610a2c565b90508181036020830152610b9f8184610afc565b90509392505050565b6000819050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b6000610bec82610ba8565b9150610bf783610ba8565b9250828201905080821115610c0f57610c0e610bb2565b5b92915050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b7f4e487b7100000000000000000000000000000000000000000000000000000000600052603260045260246000fd5b7f4e487b7100000000000000000000000000000000000000000000000000000000600052602160045260246000fd5b600080fd5b600080fd5b600080fd5b60008083356001602003843603038112610cce57610ccd610ca2565b5b80840192508235915067ffffffffffffffff821115610cf057610cef610ca7565b5b602083019250600182023603831315610d0c57610d0b610cac565b5b509250929050565b610d1d81610ba8565b82525050565b6000604082019050610d386000830185610d14565b8181036020830152610d4a8184610a2c565b90509392505050565b60007fffffffff0000000000000000000000000000000000000000000000000000000082169050919050565b6000819050919050565b610d9a610d9582610d53565b610d7f565b82525050565b600081905092915050565b82818337600083830152505050565b6000610dc68385610da0565b9350610dd3838584610dab565b82840190509392505050565b6000610deb8286610d89565b600482019150610dfc828486610dba565b9150819050949350505050565b6000610e14826109d5565b610e1e8185610da0565b9350610e2e8185602086016109f1565b80840191505092915050565b6000610e468284610e09565b915081905092915050565b6000819050919050565b610e6c610e6782610ba8565b610e51565b82525050565b6000610e7e8284610e5b565b60208201915081905092915050565b600082905092915050565b60007fffffffffffffffffffffffffffffffffffffffff00000000000000000000000082169050919050565b600082821b905092915050565b6000610edd8383610e8d565b82610ee88135610e98565b92506014821015610f2857610f237fffffffffffffffffffffffffffffffffffffffff00000000000000000000000083601403600802610ec4565b831692505b505092915050565b6000819050919050565b6000610f468383610e8d565b82610f518135610f30565b92506020821015610f9157610f8c7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff83602003600802610ec4565b831692505b505092915050565b600080fd5b600080fd5b60008085851115610fb757610fb6610f99565b5b83861115610fc857610fc7610f9e565b5b6001850283019150848603905094509492505050565b6000610feb828486610dba565b91508190509392505050565b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b600061102282610ff7565b9050919050565b60008160601b9050919050565b600061104182611029565b9050919050565b600061105382611036565b9050919050565b61106b61106682611017565b611048565b82525050565b600061107d828461105a565b6014820191508190509291505056fea2646970667358221220eab34234e1d6ca4d4dd1330b69ac489a8791baf6732478a7ebb52b151a02435164736f6c63430008130033" . parse () . expect ("invalid bytecode")
        });
    pub struct MultiCall<M>(ethers::contract::Contract<M>);
    impl<M> Clone for MultiCall<M> {
        fn clone(&self) -> Self {
            MultiCall(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for MultiCall<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for MultiCall<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(MultiCall))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> MultiCall<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), MULTICALL_ABI.clone(), client).into()
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
                MULTICALL_ABI.clone(),
                MULTICALL_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        #[doc = "Calls the contract's `multicall` (0x9171e82e) function"]
        pub fn multicall(
            &self,
            commands: ethers::core::types::Bytes,
            inputs: ::std::vec::Vec<ethers::core::types::Bytes>,
        ) -> ethers::contract::builders::ContractCall<
            M,
            (
                ethers::core::types::Bytes,
                ::std::vec::Vec<ethers::core::types::Bytes>,
            ),
        > {
            self.0
                .method_hash([145, 113, 232, 46], (commands, inputs))
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for MultiCall<M> {
        fn from(contract: ethers::contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
    #[doc = "Custom Error type `Failed` with signature `Failed(uint256,bytes)` and selector `[23, 58, 223, 132]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthError,
        ethers :: contract :: EthDisplay,
    )]
    #[etherror(name = "Failed", abi = "Failed(uint256,bytes)")]
    pub struct Failed {
        pub index: ethers::core::types::U256,
        pub data: ethers::core::types::Bytes,
    }
    #[doc = "Custom Error type `InvalidCommand` with signature `InvalidCommand()` and selector `[18, 242, 105, 229]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthError,
        ethers :: contract :: EthDisplay,
    )]
    #[etherror(name = "InvalidCommand", abi = "InvalidCommand()")]
    pub struct InvalidCommand;
    #[doc = "Custom Error type `LengthMismatch` with signature `LengthMismatch()` and selector `[255, 99, 58, 56]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthError,
        ethers :: contract :: EthDisplay,
    )]
    #[etherror(name = "LengthMismatch", abi = "LengthMismatch()")]
    pub struct LengthMismatch;
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum MultiCallErrors {
        Failed(Failed),
        InvalidCommand(InvalidCommand),
        LengthMismatch(LengthMismatch),
    }
    impl ethers::core::abi::AbiDecode for MultiCallErrors {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) = <Failed as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(MultiCallErrors::Failed(decoded));
            }
            if let Ok(decoded) =
                <InvalidCommand as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(MultiCallErrors::InvalidCommand(decoded));
            }
            if let Ok(decoded) =
                <LengthMismatch as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(MultiCallErrors::LengthMismatch(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for MultiCallErrors {
        fn encode(self) -> Vec<u8> {
            match self {
                MultiCallErrors::Failed(element) => element.encode(),
                MultiCallErrors::InvalidCommand(element) => element.encode(),
                MultiCallErrors::LengthMismatch(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for MultiCallErrors {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                MultiCallErrors::Failed(element) => element.fmt(f),
                MultiCallErrors::InvalidCommand(element) => element.fmt(f),
                MultiCallErrors::LengthMismatch(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<Failed> for MultiCallErrors {
        fn from(var: Failed) -> Self {
            MultiCallErrors::Failed(var)
        }
    }
    impl ::std::convert::From<InvalidCommand> for MultiCallErrors {
        fn from(var: InvalidCommand) -> Self {
            MultiCallErrors::InvalidCommand(var)
        }
    }
    impl ::std::convert::From<LengthMismatch> for MultiCallErrors {
        fn from(var: LengthMismatch) -> Self {
            MultiCallErrors::LengthMismatch(var)
        }
    }
    #[doc = "Container type for all input parameters for the `multicall` function with signature `multicall(bytes,bytes[])` and selector `[145, 113, 232, 46]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "multicall", abi = "multicall(bytes,bytes[])")]
    pub struct MulticallCall {
        pub commands: ethers::core::types::Bytes,
        pub inputs: ::std::vec::Vec<ethers::core::types::Bytes>,
    }
    #[doc = "Container type for all return fields from the `multicall` function with signature `multicall(bytes,bytes[])` and selector `[145, 113, 232, 46]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct MulticallReturn {
        pub successes: ethers::core::types::Bytes,
        pub outputs: ::std::vec::Vec<ethers::core::types::Bytes>,
    }
}
