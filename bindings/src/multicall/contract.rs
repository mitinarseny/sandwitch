use std::{
    fmt::{self, Debug, Display},
    sync::Arc,
};

use ethers::{
    abi::{self, AbiDecode, AbiError, InvalidOutputType},
    contract::{ContractError as RawContractError},
    providers::{Middleware, ProviderError},
    types::{Address, Bytes},
};
use thiserror::Error as ThisError;

use super::{
    multicall::{MultiCall, MultiCallErrors},
    raw,
};

// #[autoimpl(Deref<Target = Contract<M>> using self.0)]
pub struct MultiCallContract<M>(raw::MultiCall<M>);

impl<M> MultiCallContract<M> {
    pub fn address(&self) -> Address {
        self.0.address()
    }
}

impl<M> MultiCallContract<M>
where
    M: Middleware + 'static,
{
    pub fn new(address: Address, client: impl Into<Arc<M>>) -> Self {
        Self(raw::MultiCall::new(address, client.into()))
    }

    pub async fn multicall<C: MultiCall>(
        &self,
        calls: C,
    ) -> Result<C::Ok, MultiCallContractError<M, C::Reverted>> {
        let (r, meta) = calls.encode_calls_raw();
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

    pub async fn owner(&self) -> Result<Address, RawContractError<M>> {
        self.0.owner().await
    }

    pub async fn transfer_ownership(&self, new_owner: Address) -> Result<(), RawContractError<M>> {
        self.0.transfer_ownership(new_owner).await
    }
}

impl<M> Debug for MultiCallContract<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
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
