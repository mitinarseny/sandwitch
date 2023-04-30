use core::{
    borrow::Borrow,
    fmt::{self, Debug},
};

use ethers::{
    abi::AbiError,
    contract::{ContractInstance, EthCall},
    providers::Middleware,
    types::{Address, BlockId, TxHash, U256},
};
use impl_tools::autoimpl;

use crate::{
    prelude::{ContractError, TypedFunctionCall},
    EthTypedCall,
};

use super::{
    multicall::{MultiCall, MultiCallErrors},
    raw,
};

#[autoimpl(Deref using self.inner)]
pub struct MultiFunctionCall<B, M, C: MultiCall> {
    inner: TypedFunctionCall<B, M, raw::MulticallCall>,
    meta: C::Meta,
}

impl<B, M, C> MultiFunctionCall<B, M, C>
where
    B: Borrow<M>,
    M: Middleware,
    C: MultiCall,
{
    pub fn from(mut self, from: Address) -> Self {
        self.inner = self.inner.from(from);
        self
    }

    pub fn block(mut self, block: impl Into<BlockId>) -> Self {
        self.inner = self.inner.block(block);
        self
    }

    pub fn priority_fee_per_gas(mut self, priority_fee: impl Into<U256>) -> Self {
        self.inner = self.inner.priority_fee_per_gas(priority_fee);
        self
    }

    pub fn value(mut self, value: impl Into<U256>) -> Self {
        self.inner = self.inner.value(value);
        self
    }

    pub async fn call(
        &self,
    ) -> Result<Result<C::Ok, MultiCallErrors<C::Reverted>>, ContractError<M>> {
        Ok(match self.inner.call().await? {
            Ok(output) => Ok(C::decode_ok_raw(output, &self.meta)?),
            Err(reverted) => Err(self.decode_reverted(reverted)?),
        })
    }

    pub async fn estimate_gas(
        &self,
    ) -> Result<Result<U256, MultiCallErrors<C::Reverted>>, ContractError<M>> {
        Ok(match self.inner.estimate_gas().await? {
            Ok(gas) => Ok(gas),
            Err(reverted) => Err(self.decode_reverted(reverted)?),
        })
    }

    pub async fn send(&self) -> Result<TxHash, M::Error> {
        self.inner.send().await
    }

    fn decode_reverted(
        &self,
        err: <raw::MulticallCall as EthTypedCall>::Reverted,
    ) -> Result<MultiCallErrors<C::Reverted>, AbiError> {
        C::decode_reverted_raw_errors(err, &self.meta)
    }
}

// #[autoimpl(Deref<Target = Contract<M>> using self.0)]
pub struct MultiCallContract<B, M>(ContractInstance<B, M>);

impl<B, M> MultiCallContract<B, M>
where
    B: Borrow<M>,
{
    pub fn address(&self) -> Address {
        self.0.address()
    }
}

impl<B, M> MultiCallContract<B, M>
where
    B: Borrow<M> + Clone,
    M: Middleware,
{
    pub fn new(address: Address, client: B) -> Self {
        Self(ContractInstance::new(
            address,
            raw::MULTICALL_ABI.clone(),
            client,
        ))
    }

    pub fn multicall<C: MultiCall>(&self, calls: C) -> MultiFunctionCall<B, M, C> {
        let (r, meta) = calls.encode_raw_calls();
        MultiFunctionCall {
            inner: self
                .0
                .method_hash(<raw::MulticallCall>::selector(), r)
                .expect("method not found")
                .into(),
            meta,
        }
    }

    // TODO: other methods
    // pub async fn owner(&self) -> Result<Address, RawContractError<M>> {
    //     self.0.owner().await
    // }

    // pub async fn transfer_ownership(&self, new_owner: Address) -> Result<(), RawContractError<M>> {
    //     self.0.transfer_ownership(new_owner).await
    // }
}

impl<B, M> Debug for MultiCallContract<B, M>
where
    B: Debug,
    M: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
