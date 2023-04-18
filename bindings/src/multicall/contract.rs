use core::{
    borrow::Borrow,
    fmt::{self, Debug},
};

use ethers::{
    contract::{ContractInstance, EthCall},
    providers::Middleware,
    types::{transaction::eip2718::TypedTransaction, Address, BlockId, TxHash, U256},
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

    pub fn value(mut self, value: impl Into<U256>) -> Self {
        self.inner = self.inner.value(value);
        self
    }

    pub async fn call(&self) -> Result<C::Ok, MultiCallContractError<M, C::Reverted>> {
        C::decode_ok_raw(
            self.inner
                .call()
                .await
                .map_err(|e| self.decode_reverted(e))?,
            &self.meta,
        )
        .map_err(Into::into)
    }

    pub async fn estimate_gas(&self) -> Result<U256, MultiCallContractError<M, C::Reverted>> {
        self.inner
            .estimate_gas()
            .await
            .map_err(|e| self.decode_reverted(e))
    }

    pub async fn send(&self) -> Result<TxHash, M::Error> {
        self.inner.send().await
    }

    fn decode_reverted(
        &self,
        err: ContractError<M, <raw::MulticallCall as EthTypedCall>::Reverted>,
    ) -> MultiCallContractError<M, C::Reverted> {
        err.try_decode_revert_with(|r| C::decode_reverted_raw_errors(r, &self.meta))
    }

    pub fn priority_fee(mut self, priority_fee: impl Into<U256>) -> Self {
        match &mut self.inner.0.tx {
            TypedTransaction::Legacy(_) => todo!(),
            TypedTransaction::Eip2930(_) => todo!(),
            TypedTransaction::Eip1559(_) => todo!(),
        }
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

pub type MultiCallContractError<M, R> = ContractError<M, MultiCallErrors<R>>;
