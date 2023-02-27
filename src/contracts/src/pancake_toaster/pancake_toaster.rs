pub use pancake_toaster::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod pancake_toaster {
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
    #[doc = "PancakeToaster was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"contract IPancakeRouter02\",\"name\":\"_router\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"user\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"newOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"OwnershipTransferred\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountOut\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountInMax\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"path\",\"type\":\"address[]\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"deadline\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"bakeSwapETHForExactTokens\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountIn\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountOutMin\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"path\",\"type\":\"address[]\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"deadline\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"bakeSwapExactETHForTokens\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountIn\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountOutMin\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"path\",\"type\":\"address[]\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"deadline\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"bakeSwapExactTokensForTokens\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountOut\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountInMax\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"path\",\"type\":\"address[]\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"deadline\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"bakeSwapTokensForExactTokens\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"owner\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"router\",\"outputs\":[{\"internalType\":\"contract IPancakeRouter02\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"newOwner\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transferOwnership\",\"outputs\":[]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static PANCAKETOASTER_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    #[doc = r" Bytecode of the #name contract"]
    pub static PANCAKETOASTER_BYTECODE: ethers::contract::Lazy<ethers::core::types::Bytes> =
        ethers::contract::Lazy::new(|| {
            "0x60a06040523480156200001157600080fd5b506040516200136c3803806200136c83398181016040528101906200003791906200018d565b33806000806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508073ffffffffffffffffffffffffffffffffffffffff16600073ffffffffffffffffffffffffffffffffffffffff167f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e060405160405180910390a3508073ffffffffffffffffffffffffffffffffffffffff1660808173ffffffffffffffffffffffffffffffffffffffff168152505050620001bf565b600080fd5b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b6000620001418262000114565b9050919050565b6000620001558262000134565b9050919050565b620001678162000148565b81146200017357600080fd5b50565b60008151905062000187816200015c565b92915050565b600060208284031215620001a657620001a56200010f565b5b6000620001b68482850162000176565b91505092915050565b608051611183620001e960003960008181610265015281816107230152610c1701526111836000f3fe608060405234801561001057600080fd5b506004361061007d5760003560e01c80638da5cb5b1161005b5780638da5cb5b146100d6578063f2fde38b146100f4578063f6a1b90a14610110578063f887ea401461012c5761007d565b80630fc3fcf9146100825780631de0740b1461009e5780638bf20470146100ba575b600080fd5b61009c60048036038101906100979190610d48565b61014a565b005b6100b860048036038101906100b39190610d48565b6103f6565b005b6100d460048036038101906100cf9190610d48565b610608565b005b6100de6108b4565b6040516100eb9190610df1565b60405180910390f35b61010e60048036038101906101099190610e0c565b6108d8565b005b61012a60048036038101906101259190610d48565b610a03565b005b610134610c15565b6040516101419190610e98565b60405180910390f35b60008054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff16146101d8576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016101cf90610f10565b60405180910390fd5b804281101561021c576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161021390610f7c565b60405180910390fd5b6002848490501015610263576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161025a90610fe8565b60405180910390fd5b7f000000000000000000000000000000000000000000000000000000000000000073ffffffffffffffffffffffffffffffffffffffff1663ad5c46486040518163ffffffff1660e01b8152600401602060405180830381865afa1580156102ce573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906102f2919061101d565b73ffffffffffffffffffffffffffffffffffffffff168484600081811061031c5761031b61104a565b5b90506020020160208101906103319190610e0c565b73ffffffffffffffffffffffffffffffffffffffff1614610387576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161037e90610fe8565b60405180910390fd5b848773ffffffffffffffffffffffffffffffffffffffff163110156103e1576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016103d8906110eb565b60405180910390fd5b6103ed86868686610c39565b50505050505050565b60008054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff1614610484576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161047b90610f10565b60405180910390fd5b80428110156104c8576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016104bf90610f7c565b60405180910390fd5b600284849050101561050f576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161050690610fe8565b60405180910390fd5b85848460008181106105245761052361104a565b5b90506020020160208101906105399190610e0c565b73ffffffffffffffffffffffffffffffffffffffff166370a08231896040518263ffffffff1660e01b81526004016105719190610df1565b602060405180830381865afa15801561058e573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906105b29190611120565b10156105f3576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016105ea906110eb565b60405180910390fd5b6105ff86868686610c3f565b50505050505050565b60008054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff1614610696576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161068d90610f10565b60405180910390fd5b80428110156106da576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016106d190610f7c565b60405180910390fd5b6002848490501015610721576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161071890610fe8565b60405180910390fd5b7f000000000000000000000000000000000000000000000000000000000000000073ffffffffffffffffffffffffffffffffffffffff1663ad5c46486040518163ffffffff1660e01b8152600401602060405180830381865afa15801561078c573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906107b0919061101d565b73ffffffffffffffffffffffffffffffffffffffff16848460008181106107da576107d961104a565b5b90506020020160208101906107ef9190610e0c565b73ffffffffffffffffffffffffffffffffffffffff1614610845576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161083c90610fe8565b60405180910390fd5b858773ffffffffffffffffffffffffffffffffffffffff1631101561089f576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610896906110eb565b60405180910390fd5b6108ab86868686610c3f565b50505050505050565b60008054906101000a900473ffffffffffffffffffffffffffffffffffffffff1681565b60008054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff1614610966576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161095d90610f10565b60405180910390fd5b806000806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508073ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff167f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e060405160405180910390a350565b60008054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff1614610a91576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610a8890610f10565b60405180910390fd5b8042811015610ad5576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610acc90610f7c565b60405180910390fd5b6002848490501015610b1c576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610b1390610fe8565b60405180910390fd5b8484846000818110610b3157610b3061104a565b5b9050602002016020810190610b469190610e0c565b73ffffffffffffffffffffffffffffffffffffffff166370a08231896040518263ffffffff1660e01b8152600401610b7e9190610df1565b602060405180830381865afa158015610b9b573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610bbf9190611120565b1015610c00576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610bf7906110eb565b60405180910390fd5b610c0c86868686610c39565b50505050505050565b7f000000000000000000000000000000000000000000000000000000000000000081565b50505050565b50505050565b600080fd5b600080fd5b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b6000610c7a82610c4f565b9050919050565b610c8a81610c6f565b8114610c9557600080fd5b50565b600081359050610ca781610c81565b92915050565b6000819050919050565b610cc081610cad565b8114610ccb57600080fd5b50565b600081359050610cdd81610cb7565b92915050565b600080fd5b600080fd5b600080fd5b60008083601f840112610d0857610d07610ce3565b5b8235905067ffffffffffffffff811115610d2557610d24610ce8565b5b602083019150836020820283011115610d4157610d40610ced565b5b9250929050565b60008060008060008060a08789031215610d6557610d64610c45565b5b6000610d7389828a01610c98565b9650506020610d8489828a01610cce565b9550506040610d9589828a01610cce565b945050606087013567ffffffffffffffff811115610db657610db5610c4a565b5b610dc289828a01610cf2565b93509350506080610dd589828a01610cce565b9150509295509295509295565b610deb81610c6f565b82525050565b6000602082019050610e066000830184610de2565b92915050565b600060208284031215610e2257610e21610c45565b5b6000610e3084828501610c98565b91505092915050565b6000819050919050565b6000610e5e610e59610e5484610c4f565b610e39565b610c4f565b9050919050565b6000610e7082610e43565b9050919050565b6000610e8282610e65565b9050919050565b610e9281610e77565b82525050565b6000602082019050610ead6000830184610e89565b92915050565b600082825260208201905092915050565b7f554e415554484f52495a45440000000000000000000000000000000000000000600082015250565b6000610efa600c83610eb3565b9150610f0582610ec4565b602082019050919050565b60006020820190508181036000830152610f2981610eed565b9050919050565b7f50616e63616b65546f61737465723a2045585049524544000000000000000000600082015250565b6000610f66601783610eb3565b9150610f7182610f30565b602082019050919050565b60006020820190508181036000830152610f9581610f59565b9050919050565b7f50616e63616b65546f61737465723a20494e56414c49445f5041544800000000600082015250565b6000610fd2601c83610eb3565b9150610fdd82610f9c565b602082019050919050565b6000602082019050818103600083015261100181610fc5565b9050919050565b60008151905061101781610c81565b92915050565b60006020828403121561103357611032610c45565b5b600061104184828501611008565b91505092915050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052603260045260246000fd5b7f50616e63616b65546f61737465723a20494e53554646494349454e545f42414c60008201527f414e434500000000000000000000000000000000000000000000000000000000602082015250565b60006110d5602483610eb3565b91506110e082611079565b604082019050919050565b60006020820190508181036000830152611104816110c8565b9050919050565b60008151905061111a81610cb7565b92915050565b60006020828403121561113657611135610c45565b5b60006111448482850161110b565b9150509291505056fea26469706673582212202feecf449cfc3c4f1ed3a5c28c59fc73e45bf0b26eddbcf0df3cf47e0acd2cbb64736f6c63430008110033" . parse () . expect ("invalid bytecode")
        });
    pub struct PancakeToaster<M>(ethers::contract::Contract<M>);
    impl<M> Clone for PancakeToaster<M> {
        fn clone(&self) -> Self {
            PancakeToaster(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for PancakeToaster<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for PancakeToaster<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(PancakeToaster))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> PancakeToaster<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), PANCAKETOASTER_ABI.clone(), client)
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
                PANCAKETOASTER_ABI.clone(),
                PANCAKETOASTER_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        #[doc = "Calls the contract's `bakeSwapETHForExactTokens` (0x0fc3fcf9) function"]
        pub fn bake_swap_eth_for_exact_tokens(
            &self,
            from: ethers::core::types::Address,
            amount_out: ethers::core::types::U256,
            amount_in_max: ethers::core::types::U256,
            path: ::std::vec::Vec<ethers::core::types::Address>,
            deadline: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [15, 195, 252, 249],
                    (from, amount_out, amount_in_max, path, deadline),
                )
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `bakeSwapExactETHForTokens` (0x8bf20470) function"]
        pub fn bake_swap_exact_eth_for_tokens(
            &self,
            from: ethers::core::types::Address,
            amount_in: ethers::core::types::U256,
            amount_out_min: ethers::core::types::U256,
            path: ::std::vec::Vec<ethers::core::types::Address>,
            deadline: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [139, 242, 4, 112],
                    (from, amount_in, amount_out_min, path, deadline),
                )
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `bakeSwapExactTokensForTokens` (0x1de0740b) function"]
        pub fn bake_swap_exact_tokens_for_tokens(
            &self,
            from: ethers::core::types::Address,
            amount_in: ethers::core::types::U256,
            amount_out_min: ethers::core::types::U256,
            path: ::std::vec::Vec<ethers::core::types::Address>,
            deadline: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [29, 224, 116, 11],
                    (from, amount_in, amount_out_min, path, deadline),
                )
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `bakeSwapTokensForExactTokens` (0xf6a1b90a) function"]
        pub fn bake_swap_tokens_for_exact_tokens(
            &self,
            from: ethers::core::types::Address,
            amount_out: ethers::core::types::U256,
            amount_in_max: ethers::core::types::U256,
            path: ::std::vec::Vec<ethers::core::types::Address>,
            deadline: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [246, 161, 185, 10],
                    (from, amount_out, amount_in_max, path, deadline),
                )
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `owner` (0x8da5cb5b) function"]
        pub fn owner(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([141, 165, 203, 91], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `router` (0xf887ea40) function"]
        pub fn router(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([248, 135, 234, 64], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `transferOwnership` (0xf2fde38b) function"]
        pub fn transfer_ownership(
            &self,
            new_owner: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([242, 253, 227, 139], new_owner)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Gets the contract's `OwnershipTransferred` event"]
        pub fn ownership_transferred_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, OwnershipTransferredFilter> {
            self.0.event()
        }
        #[doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(&self) -> ethers::contract::builders::Event<M, OwnershipTransferredFilter> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for PancakeToaster<M> {
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
        name = "OwnershipTransferred",
        abi = "OwnershipTransferred(address,address)"
    )]
    pub struct OwnershipTransferredFilter {
        #[ethevent(indexed)]
        pub user: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub new_owner: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `bakeSwapETHForExactTokens` function with signature `bakeSwapETHForExactTokens(address,uint256,uint256,address[],uint256)` and selector `[15, 195, 252, 249]`"]
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
        name = "bakeSwapETHForExactTokens",
        abi = "bakeSwapETHForExactTokens(address,uint256,uint256,address[],uint256)"
    )]
    pub struct BakeSwapETHForExactTokensCall {
        pub from: ethers::core::types::Address,
        pub amount_out: ethers::core::types::U256,
        pub amount_in_max: ethers::core::types::U256,
        pub path: ::std::vec::Vec<ethers::core::types::Address>,
        pub deadline: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `bakeSwapExactETHForTokens` function with signature `bakeSwapExactETHForTokens(address,uint256,uint256,address[],uint256)` and selector `[139, 242, 4, 112]`"]
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
        name = "bakeSwapExactETHForTokens",
        abi = "bakeSwapExactETHForTokens(address,uint256,uint256,address[],uint256)"
    )]
    pub struct BakeSwapExactETHForTokensCall {
        pub from: ethers::core::types::Address,
        pub amount_in: ethers::core::types::U256,
        pub amount_out_min: ethers::core::types::U256,
        pub path: ::std::vec::Vec<ethers::core::types::Address>,
        pub deadline: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `bakeSwapExactTokensForTokens` function with signature `bakeSwapExactTokensForTokens(address,uint256,uint256,address[],uint256)` and selector `[29, 224, 116, 11]`"]
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
        name = "bakeSwapExactTokensForTokens",
        abi = "bakeSwapExactTokensForTokens(address,uint256,uint256,address[],uint256)"
    )]
    pub struct BakeSwapExactTokensForTokensCall {
        pub from: ethers::core::types::Address,
        pub amount_in: ethers::core::types::U256,
        pub amount_out_min: ethers::core::types::U256,
        pub path: ::std::vec::Vec<ethers::core::types::Address>,
        pub deadline: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `bakeSwapTokensForExactTokens` function with signature `bakeSwapTokensForExactTokens(address,uint256,uint256,address[],uint256)` and selector `[246, 161, 185, 10]`"]
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
        name = "bakeSwapTokensForExactTokens",
        abi = "bakeSwapTokensForExactTokens(address,uint256,uint256,address[],uint256)"
    )]
    pub struct BakeSwapTokensForExactTokensCall {
        pub from: ethers::core::types::Address,
        pub amount_out: ethers::core::types::U256,
        pub amount_in_max: ethers::core::types::U256,
        pub path: ::std::vec::Vec<ethers::core::types::Address>,
        pub deadline: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `owner` function with signature `owner()` and selector `[141, 165, 203, 91]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "owner", abi = "owner()")]
    pub struct OwnerCall;
    #[doc = "Container type for all input parameters for the `router` function with signature `router()` and selector `[248, 135, 234, 64]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "router", abi = "router()")]
    pub struct RouterCall;
    #[doc = "Container type for all input parameters for the `transferOwnership` function with signature `transferOwnership(address)` and selector `[242, 253, 227, 139]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "transferOwnership", abi = "transferOwnership(address)")]
    pub struct TransferOwnershipCall {
        pub new_owner: ethers::core::types::Address,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum PancakeToasterCalls {
        BakeSwapETHForExactTokens(BakeSwapETHForExactTokensCall),
        BakeSwapExactETHForTokens(BakeSwapExactETHForTokensCall),
        BakeSwapExactTokensForTokens(BakeSwapExactTokensForTokensCall),
        BakeSwapTokensForExactTokens(BakeSwapTokensForExactTokensCall),
        Owner(OwnerCall),
        Router(RouterCall),
        TransferOwnership(TransferOwnershipCall),
    }
    impl ethers::core::abi::AbiDecode for PancakeToasterCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <BakeSwapETHForExactTokensCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(PancakeToasterCalls::BakeSwapETHForExactTokens(decoded));
            }
            if let Ok(decoded) =
                <BakeSwapExactETHForTokensCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(PancakeToasterCalls::BakeSwapExactETHForTokens(decoded));
            }
            if let Ok(decoded) =
                <BakeSwapExactTokensForTokensCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(PancakeToasterCalls::BakeSwapExactTokensForTokens(decoded));
            }
            if let Ok(decoded) =
                <BakeSwapTokensForExactTokensCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(PancakeToasterCalls::BakeSwapTokensForExactTokens(decoded));
            }
            if let Ok(decoded) = <OwnerCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeToasterCalls::Owner(decoded));
            }
            if let Ok(decoded) = <RouterCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeToasterCalls::Router(decoded));
            }
            if let Ok(decoded) =
                <TransferOwnershipCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeToasterCalls::TransferOwnership(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for PancakeToasterCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                PancakeToasterCalls::BakeSwapETHForExactTokens(element) => element.encode(),
                PancakeToasterCalls::BakeSwapExactETHForTokens(element) => element.encode(),
                PancakeToasterCalls::BakeSwapExactTokensForTokens(element) => element.encode(),
                PancakeToasterCalls::BakeSwapTokensForExactTokens(element) => element.encode(),
                PancakeToasterCalls::Owner(element) => element.encode(),
                PancakeToasterCalls::Router(element) => element.encode(),
                PancakeToasterCalls::TransferOwnership(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for PancakeToasterCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                PancakeToasterCalls::BakeSwapETHForExactTokens(element) => element.fmt(f),
                PancakeToasterCalls::BakeSwapExactETHForTokens(element) => element.fmt(f),
                PancakeToasterCalls::BakeSwapExactTokensForTokens(element) => element.fmt(f),
                PancakeToasterCalls::BakeSwapTokensForExactTokens(element) => element.fmt(f),
                PancakeToasterCalls::Owner(element) => element.fmt(f),
                PancakeToasterCalls::Router(element) => element.fmt(f),
                PancakeToasterCalls::TransferOwnership(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<BakeSwapETHForExactTokensCall> for PancakeToasterCalls {
        fn from(var: BakeSwapETHForExactTokensCall) -> Self {
            PancakeToasterCalls::BakeSwapETHForExactTokens(var)
        }
    }
    impl ::std::convert::From<BakeSwapExactETHForTokensCall> for PancakeToasterCalls {
        fn from(var: BakeSwapExactETHForTokensCall) -> Self {
            PancakeToasterCalls::BakeSwapExactETHForTokens(var)
        }
    }
    impl ::std::convert::From<BakeSwapExactTokensForTokensCall> for PancakeToasterCalls {
        fn from(var: BakeSwapExactTokensForTokensCall) -> Self {
            PancakeToasterCalls::BakeSwapExactTokensForTokens(var)
        }
    }
    impl ::std::convert::From<BakeSwapTokensForExactTokensCall> for PancakeToasterCalls {
        fn from(var: BakeSwapTokensForExactTokensCall) -> Self {
            PancakeToasterCalls::BakeSwapTokensForExactTokens(var)
        }
    }
    impl ::std::convert::From<OwnerCall> for PancakeToasterCalls {
        fn from(var: OwnerCall) -> Self {
            PancakeToasterCalls::Owner(var)
        }
    }
    impl ::std::convert::From<RouterCall> for PancakeToasterCalls {
        fn from(var: RouterCall) -> Self {
            PancakeToasterCalls::Router(var)
        }
    }
    impl ::std::convert::From<TransferOwnershipCall> for PancakeToasterCalls {
        fn from(var: TransferOwnershipCall) -> Self {
            PancakeToasterCalls::TransferOwnership(var)
        }
    }
    #[doc = "Container type for all return fields from the `owner` function with signature `owner()` and selector `[141, 165, 203, 91]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct OwnerReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `router` function with signature `router()` and selector `[248, 135, 234, 64]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct RouterReturn(pub ethers::core::types::Address);
}
