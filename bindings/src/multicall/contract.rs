use ethers::{
    abi::{self, AbiDecode, AbiError, InvalidOutputType},
    contract::{Contract, ContractError as RawContractError},
    providers::{Middleware, ProviderError},
    types::Bytes,
};
use impl_tools::autoimpl;
use thiserror::Error as ThisError;

use super::{raw, multicall::{MultiCall, MultiCallErrors}};

#[autoimpl(Deref<Target = Contract<M>> using self.0)]
pub struct MultiCallContract<M: Middleware>(raw::MultiCall<M>);

impl<M: Middleware> MultiCallContract<M> {
    pub async fn multicall<C: MultiCall>(
        &self,
        calls: C,
    ) -> Result<C::Ok, MultiCallContractError<M, C::Reverted>> {
        let (r, meta) = calls.encode_raw();
        match self.0.multicall(r.commands, r.inputs).call().await {
            Ok((successes, outputs)) => {
                C::decode_ok_raw(raw::MulticallReturn { successes, outputs }, meta)
                    .map_err(Into::into)
            }
            Err(r) => Err(match r {
                RawContractError::DecodingError(e) => ContractError::DecodingError(e),
                RawContractError::AbiError(e) => ContractError::AbiError(e),
                RawContractError::DetokenizationError(e) => ContractError::DetokenizationError(e),
                RawContractError::MiddlewareError { e } => ContractError::MiddlewareError(e),
                RawContractError::ProviderError { e } => ContractError::ProviderError(e),
                RawContractError::Revert(Bytes(data)) => ContractError::Revert(
                    C::decode_reverted_raw_errors(AbiDecode::decode(data)?, meta)?,
                ),
                RawContractError::ConstructorError => ContractError::ConstructorError,
                RawContractError::ContractNotDeployed => ContractError::ContractNotDeployed,
            }),
        }
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

pub type MultiCallContractError<M, R> = ContractError<M, MultiCallErrors<R>>;
