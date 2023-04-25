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
                env!("CARGO_MANIFEST_DIR"), "/../", $path));
            ::ethers::contract::abigen!($name, $path $(, $other)*);
        };
    }
    pub(crate) use tracked_abigen;

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
    pub enum ContractError<M: Middleware, R> {
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
        ProviderError(ProviderError),

        /// Contract reverted
        #[error("contract call reverted with: {0}")]
        Revert(R),

        /// Thrown during deployment if a constructor argument was passed in the `deploy`
        /// call but a constructor was not present in the ABI
        #[error("constructor is not defined in the ABI")]
        ConstructorError,

        /// Thrown if a contract address is not found in the deployment transaction's
        /// receipt
        #[error("Contract was not deployed")]
        ContractNotDeployed,
    }

    impl<M, R> ContractError<M, R>
    where
        M: Middleware,
    {
        pub(crate) fn try_decode_revert_with<R2>(
            self,
            f: impl FnOnce(R) -> Result<R2, AbiError>,
        ) -> ContractError<M, R2> {
            match self {
                Self::DecodingError(e) => ContractError::DecodingError(e),
                Self::AbiError(e) => ContractError::AbiError(e),
                Self::DetokenizationError(e) => ContractError::DetokenizationError(e),
                Self::MiddlewareError(e) => ContractError::MiddlewareError(e),
                Self::ProviderError(e) => ContractError::ProviderError(e),
                Self::Revert(e) => f(e)
                    .map(ContractError::Revert)
                    .unwrap_or_else(ContractError::AbiError),
                Self::ConstructorError => ContractError::ConstructorError,
                Self::ContractNotDeployed => ContractError::ContractNotDeployed,
            }
        }
    }

    impl<M, R> From<RawContractError<M>> for ContractError<M, R>
    where
        M: Middleware,
        R: AbiDecode,
    {
        fn from(err: RawContractError<M>) -> Self {
            match err {
                RawContractError::DecodingError(e) => Self::DecodingError(e),
                RawContractError::AbiError(e) => Self::AbiError(e),
                RawContractError::DetokenizationError(e) => Self::DetokenizationError(e),
                RawContractError::MiddlewareError { e } => Self::MiddlewareError(e),
                RawContractError::ProviderError { e } => Self::ProviderError(e),
                RawContractError::Revert(Bytes(data)) => {
                    R::decode(data).map(Self::Revert).unwrap_or_else(Into::into)
                }
                RawContractError::ConstructorError => Self::ConstructorError,
                RawContractError::ContractNotDeployed => Self::ContractNotDeployed,
            }
        }
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

        pub async fn call(&self) -> Result<C::Ok, ContractError<M, C::Reverted>> {
            self.0.call().await.map_err(Into::into)
        }

        pub async fn estimate_gas(&self) -> Result<U256, ContractError<M, C::Reverted>> {
            self.0.estimate_gas().await.map_err(Into::into)
        }

        pub async fn send(&self) -> Result<TxHash, M::Error> {
            let pending_tx = self.0.send().await.map_err(|e| match e {
                RawContractError::MiddlewareError { e } => e,
                _ => unreachable!(),
            })?;
            Ok(pending_tx.tx_hash())
        }
    }
}
pub use prelude::EthTypedCall;

// #[cfg(feature = "erc20")]
pub mod erc20 {
    use crate::prelude::*;
    tracked_abigen!(ERC20, "contracts/out/ERC20.sol/ERC20.json");

    impl EthTypedCall for ApproveCall {
        type Ok = maybe::OkOrNone;
        type Reverted = RawReverted;
    }
}

#[cfg(feature = "multicall")]
pub mod multicall;

#[cfg(feature = "pancake_swap")]
pub mod pancake_swap {
    use crate::prelude::*;
    tracked_abigen!(
        PancakeRouter,
        "contracts/out/IPancakeRouter02.sol/IPancakeRouter02.json"
    );
}

#[cfg(feature = "pancake_toaster")]
pub mod pancake_toaster {
    use crate::prelude::*;
    tracked_abigen!(
        PancakeToaster,
        "contracts/out/PancakeToaster.sol/PancakeToaster.json"
    );

    impl EthTypedCall for FrontRunSwapCall {
        type Ok = FrontRunSwapReturn;
        type Reverted = PancakeToasterErrors;
    }
}
