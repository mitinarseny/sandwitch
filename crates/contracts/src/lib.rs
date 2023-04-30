#![feature(iterator_try_collect)]

pub(crate) mod utils;

pub(crate) mod prelude {
    use std::borrow::Borrow;

    use ethers::{
        abi::{self, AbiDecode, AbiEncode, AbiError, InvalidOutputType, Tokenizable},
        contract::{ContractError as RawContractError, ContractRevert, EthCall, FunctionCall},
        providers::{Middleware, ProviderError},
        types::{
            transaction::eip2718::TypedTransaction, Address, BlockId, Bytes, Selector, TxHash, U256,
        },
    };
    use thiserror::Error as ThisError;

    pub use crate::utils::*;

    #[allow(unused_macros)]
    macro_rules! tracked_abigen {
        ($name:ident, $path:literal $(, $other:expr)*) => {
            const _: &'static str = include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"), "/../../", $path));
            ::ethers::contract::abigen!($name, $path $(, $other)*);
        };
    }
    pub(crate) use tracked_abigen;

    #[derive(ThisError, Debug)]
    #[error("{0}")]
    pub struct RawReverted(String);

    impl AbiEncode for RawReverted {
        fn encode(self) -> Vec<u8> {
            self.0.encode()
        }
    }

    impl AbiDecode for RawReverted {
        fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, AbiError> {
            String::decode(bytes).map(Self)
        }
    }

    impl ContractRevert for RawReverted {
        fn valid_selector(_selector: Selector) -> bool {
            true
        }
    }

    pub trait EthTypedCall: EthCall {
        type Ok: AbiEncode + AbiDecode + Tokenizable;
        type Reverted: ContractRevert;

        fn encode_calldata(self) -> Vec<u8> {
            [Self::selector().as_slice(), self.encode().as_slice()].concat()
        }
    }

    #[derive(ThisError, Debug)]
    pub enum ContractError<M: Middleware> {
        /// Thrown when the ABI decoding fails
        #[error(transparent)]
        DecodingError(#[from] abi::Error),

        /// Thrown when the internal BaseContract errors
        #[error(transparent)]
        AbiError(#[from] AbiError),

        /// Thrown when detokenizing an argument
        #[error(transparent)]
        DetokenizationError(#[from] InvalidOutputType),

        /// Thrown when a middleware call fails
        #[error(transparent)]
        MiddlewareError(M::Error),

        /// Thrown when a provider call fails
        #[error(transparent)]
        ProviderError(#[from] ProviderError),

        /// Thrown during deployment if a constructor argument was passed in the `deploy`
        /// call but a constructor was not present in the ABI
        #[error("constructor is not defined in the ABI")]
        ConstructorError,

        /// Thrown if a contract address is not found in the deployment transaction's
        /// receipt
        #[error("Contract was not deployed")]
        ContractNotDeployed,
    }

    pub struct TypedFunctionCall<B, M, C: EthTypedCall>(pub(crate) FunctionCall<B, M, C::Ok>);

    impl<B, M, C: EthTypedCall> From<FunctionCall<B, M, C::Ok>> for TypedFunctionCall<B, M, C> {
        fn from(c: FunctionCall<B, M, C::Ok>) -> Self {
            Self(c)
        }
    }

    impl<B, M, C> TypedFunctionCall<B, M, C>
    where
        B: Borrow<M>,
        M: Middleware,
        C: EthTypedCall,
    {
        pub fn from(mut self, from: Address) -> Self {
            self.0 = self.0.from(from);
            self
        }

        pub fn block(mut self, block: impl Into<BlockId>) -> Self {
            self.0 = self.0.block(block);
            self
        }

        pub fn priority_fee_per_gas(mut self, priority_fee_per_gas: impl Into<U256>) -> Self {
            self.0.tx = match self.0.tx {
                #[cfg(not(feature = "legacy"))]
                TypedTransaction::Eip1559(tx) => tx.max_priority_fee_per_gas(priority_fee_per_gas),
                #[cfg(feature = "legacy")]
                TypedTransaction::Legacy(tx) => tx.gas_price(priority_fee_per_gas),
                _ => unreachable!(),
            }
            .into();
            self
        }

        pub fn value(mut self, value: impl Into<U256>) -> Self {
            self.0 = self.0.value(value);
            self
        }

        pub async fn call(&self) -> Result<Result<C::Ok, C::Reverted>, ContractError<M>> {
            Ok(match self.0.call().await {
                Ok(output) => Ok(output),
                Err(err) => Err(Self::try_decode_reverted(err)?),
            })
        }

        pub async fn estimate_gas(&self) -> Result<Result<U256, C::Reverted>, ContractError<M>> {
            Ok(match self.0.estimate_gas().await {
                Ok(gas) => Ok(gas),
                Err(err) => Err(Self::try_decode_reverted(err)?),
            })
        }

        pub async fn send(&self) -> Result<TxHash, M::Error> {
            let pending_tx = self.0.send().await.map_err(|e| match e {
                RawContractError::MiddlewareError { e } => e,
                _ => unreachable!(), // TODO
            })?;
            Ok(pending_tx.tx_hash())
        }

        fn try_decode_reverted(err: RawContractError<M>) -> Result<C::Reverted, ContractError<M>> {
            Err(match err {
                RawContractError::DecodingError(e) => ContractError::DecodingError(e),
                RawContractError::AbiError(e) => ContractError::AbiError(e),
                RawContractError::DetokenizationError(e) => ContractError::DetokenizationError(e),
                RawContractError::MiddlewareError { e } => ContractError::MiddlewareError(e),
                RawContractError::ProviderError { e } => ContractError::ProviderError(e),
                RawContractError::Revert(Bytes(data)) => {
                    return Ok(<C::Reverted as AbiDecode>::decode(data)?)
                }
                RawContractError::ConstructorError => ContractError::ConstructorError,
                RawContractError::ContractNotDeployed => ContractError::ContractNotDeployed,
            })
        }
    }
}
pub use self::prelude::{ContractError, EthTypedCall};

#[cfg(feature = "multicall")]
pub mod multicall;

#[cfg(feature = "erc20")]
pub mod erc20;

#[cfg(feature = "pancake_swap")]
pub mod pancake_swap;

#[cfg(feature = "pancake_toaster")]
pub mod pancake_toaster;
