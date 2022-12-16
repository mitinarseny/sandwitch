use std::{marker::PhantomData, sync::Arc};

use ethers::{
    contract::EthCall,
    types::{Block, Transaction, H256},
};
use futures::{
    future::{self, try_join, try_join_all, BoxFuture},
    FutureExt, TryFutureExt,
};

#[cfg(feature = "pancake_swap")]
pub mod pancake_swap;

pub(crate) trait TxMonitor {
    type Ok;
    type Error;

    fn process_tx<'a>(
        &'a self,
        tx: &'a Transaction,
        block_hash: H256,
    ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>>;
}

pub(crate) trait TxMonitorExt: TxMonitor {
    fn map<F, T>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Ok) -> T + Sync,
    {
        Map { inner: self, f }
    }

    fn map_err<F, E>(self, f: F) -> MapErr<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Error) -> E + Sync,
    {
        MapErr { inner: self, f }
    }

    fn err_into<E>(self) -> ErrInto<Self, E>
    where
        Self: Sized,
        Self::Error: Into<E>,
    {
        ErrInto {
            inner: self,
            _into: PhantomData,
        }
    }
}

impl<M> TxMonitorExt for M where M: TxMonitor {}

pub(crate) trait BlockMonitor {
    type Ok;
    type Error;

    fn process_block<'a>(
        &'a self,
        block: &'a Block<Transaction>,
    ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>>;
}

pub(crate) trait BlockMonitorExt: BlockMonitor {
    fn map<F, T>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Ok) -> T + Sync,
    {
        Map { inner: self, f }
    }

    fn map_err<F, E>(self, f: F) -> MapErr<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Error) -> E + Sync,
    {
        MapErr { inner: self, f }
    }

    fn err_into<E>(self) -> ErrInto<Self, E>
    where
        Self: Sized,
        Self::Error: Into<E>,
    {
        ErrInto {
            inner: self,
            _into: PhantomData,
        }
    }
}

impl<M> BlockMonitorExt for M where M: BlockMonitor {}

pub(crate) struct Noop<E: Send>(PhantomData<E>);

impl<E: Send> Default for Noop<E> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<E: Send> TxMonitor for Noop<E> {
    type Ok = ();
    type Error = E;

    fn process_tx<'a>(
        &'a self,
        _tx: &'a Transaction,
        _block_hash: H256,
    ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
        future::ok(()).boxed()
    }
}

impl<E: Send> BlockMonitor for Noop<E> {
    type Ok = ();
    type Error = E;

    fn process_block<'a>(
        &'a self,
        _block: &'a Block<Transaction>,
    ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
        future::ok(()).boxed()
    }
}

impl<M: ?Sized> TxMonitor for Box<M>
where
    M: TxMonitor,
{
    type Ok = M::Ok;
    type Error = M::Error;

    fn process_tx<'a>(
        &'a self,
        tx: &'a Transaction,
        block_hash: H256,
    ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
        (**self).process_tx(tx, block_hash)
    }
}

impl<M: ?Sized> BlockMonitor for Box<M>
where
    M: BlockMonitor,
{
    type Ok = M::Ok;
    type Error = M::Error;

    fn process_block<'a>(
        &'a self,
        block: &'a Block<Transaction>,
    ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
        (**self).process_block(block)
    }
}

impl<M: ?Sized> TxMonitor for Arc<M>
where
    M: TxMonitor,
{
    type Ok = M::Ok;
    type Error = M::Error;

    fn process_tx<'a>(
        &'a self,
        tx: &'a Transaction,
        block_hash: H256,
    ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
        (**self).process_tx(tx, block_hash)
    }
}

impl<M: ?Sized> BlockMonitor for Arc<M>
where
    M: BlockMonitor,
{
    type Ok = M::Ok;
    type Error = M::Error;

    fn process_block<'a>(
        &'a self,
        block: &'a Block<Transaction>,
    ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
        (**self).process_block(block)
    }
}

impl<M> TxMonitor for [M]
where
    M: TxMonitor,
    M::Ok: Send,
    M::Error: Send,
{
    type Ok = Vec<M::Ok>;
    type Error = M::Error;

    fn process_tx<'a>(
        &'a self,
        tx: &'a Transaction,
        block_hash: H256,
    ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
        try_join_all(self.iter().map(|m| m.process_tx(tx, block_hash))).boxed()
    }
}

impl<M> BlockMonitor for [M]
where
    M: BlockMonitor,
    M::Ok: Send,
    M::Error: Send,
{
    type Ok = Vec<M::Ok>;
    type Error = M::Error;

    fn process_block<'a>(
        &'a self,
        block: &'a Block<Transaction>,
    ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
        try_join_all(self.iter().map(|m| m.process_block(block))).boxed()
    }
}

impl<M> TxMonitor for Vec<M>
where
    M: TxMonitor,
    M::Ok: Send,
    M::Error: Send,
{
    type Ok = Vec<M::Ok>;
    type Error = M::Error;

    fn process_tx<'a>(
        &'a self,
        tx: &'a Transaction,
        block_hash: H256,
    ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
        (**self).process_tx(tx, block_hash)
    }
}

impl<M> BlockMonitor for Vec<M>
where
    M: BlockMonitor,
    M::Ok: Send,
    M::Error: Send,
{
    type Ok = Vec<M::Ok>;
    type Error = M::Error;

    fn process_block<'a>(
        &'a self,
        block: &'a Block<Transaction>,
    ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
        (**self).process_block(block)
    }
}

pub(crate) trait FunctionCallMonitor<'a, C: EthCall + 'a> {
    type Ok;
    type Error;

    fn process_func(
        &'a self,
        tx: &'a Transaction,
        block_hash: H256,
        inputs: C,
    ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>>;

    fn maybe_process_func_raw(
        &'a self,
        tx: &'a Transaction,
        block_hash: H256,
    ) -> Option<BoxFuture<'a, Result<Self::Ok, Self::Error>>> {
        let inputs = C::decode(&tx.input).ok()?;
        Some(self.process_func(tx, block_hash, inputs))
    }
}

pub struct Map<M, F> {
    inner: M,
    f: F,
}

impl<M, F, T> TxMonitor for Map<M, F>
where
    M: TxMonitor,
    F: Fn(M::Ok) -> T + Sync,
{
    type Ok = T;
    type Error = M::Error;

    fn process_tx<'a>(
        &'a self,
        tx: &'a Transaction,
        block_hash: H256,
    ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
        self.inner
            .process_tx(tx, block_hash)
            .map_ok(&self.f)
            .boxed()
    }
}

impl<M, F, T> BlockMonitor for Map<M, F>
where
    M: BlockMonitor,
    F: Fn(M::Ok) -> T + Sync,
{
    type Ok = T;
    type Error = M::Error;

    fn process_block<'a>(
        &'a self,
        block: &'a Block<Transaction>,
    ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
        self.inner.process_block(block).map_ok(&self.f).boxed()
    }
}

impl<'a, M, C, F, T> FunctionCallMonitor<'a, C> for Map<M, F>
where
    C: EthCall + 'a,
    M: FunctionCallMonitor<'a, C>,
    F: Fn(M::Ok) -> T + Sync,
{
    type Ok = T;
    type Error = M::Error;

    fn process_func(
        &'a self,
        tx: &'a Transaction,
        block_hash: H256,
        inputs: C,
    ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
        self.inner
            .process_func(tx, block_hash, inputs)
            .map_ok(&self.f)
            .boxed()
    }
}

pub(crate) struct MapErr<M, F> {
    inner: M,
    f: F,
}

impl<M, F, E> TxMonitor for MapErr<M, F>
where
    M: TxMonitor,
    F: Fn(M::Error) -> E + Sync,
{
    type Ok = M::Ok;
    type Error = E;

    fn process_tx<'a>(
        &'a self,
        tx: &'a Transaction,
        block_hash: H256,
    ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
        self.inner
            .process_tx(tx, block_hash)
            .map_err(&self.f)
            .boxed()
    }
}

impl<M, F, E> BlockMonitor for MapErr<M, F>
where
    M: BlockMonitor,
    F: Fn(M::Error) -> E + Sync,
{
    type Ok = M::Ok;
    type Error = E;

    fn process_block<'a>(
        &'a self,
        block: &'a Block<Transaction>,
    ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
        self.inner.process_block(block).map_err(&self.f).boxed()
    }
}

impl<'a, M, C, F, E> FunctionCallMonitor<'a, C> for MapErr<M, F>
where
    C: EthCall + 'a,
    M: FunctionCallMonitor<'a, C>,
    F: Fn(M::Error) -> E + Sync,
{
    type Ok = M::Ok;
    type Error = E;

    fn process_func(
        &'a self,
        tx: &'a Transaction,
        block_hash: H256,
        inputs: C,
    ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
        self.inner
            .process_func(tx, block_hash, inputs)
            .map_err(&self.f)
            .boxed()
    }
}

pub(crate) struct ErrInto<M, E> {
    inner: M,
    _into: PhantomData<E>,
}

impl<M, E> TxMonitor for ErrInto<M, E>
where
    M: TxMonitor,
    M::Error: Into<E>,
{
    type Ok = M::Ok;
    type Error = E;

    fn process_tx<'a>(
        &'a self,
        tx: &'a Transaction,
        block_hash: H256,
    ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
        self.inner.process_tx(tx, block_hash).err_into().boxed()
    }
}

impl<M, E> BlockMonitor for ErrInto<M, E>
where
    M: BlockMonitor,
    M::Error: Into<E>,
{
    type Ok = M::Ok;
    type Error = E;

    fn process_block<'a>(
        &'a self,
        block: &'a Block<Transaction>,
    ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
        self.inner.process_block(block).err_into().boxed()
    }
}

impl<'a, M, C, E> FunctionCallMonitor<'a, C> for ErrInto<M, E>
where
    C: EthCall + 'a,
    M: FunctionCallMonitor<'a, C>,
    M::Error: Into<E>,
{
    type Ok = M::Ok;
    type Error = E;

    fn process_func(
        &'a self,
        tx: &'a Transaction,
        block_hash: H256,
        inputs: C,
    ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
        self.inner
            .process_func(tx, block_hash, inputs)
            .err_into()
            .boxed()
    }
}

impl<E, M1, M2> TxMonitor for (M1, M2)
where
    E: Send + 'static,
    M1: TxMonitor<Error = E>,
    M1::Ok: Send,
    M2: TxMonitor<Error = E>,
    M2::Ok: Send,
{
    type Ok = (M1::Ok, M2::Ok);

    type Error = E;

    fn process_tx<'a>(
        &'a self,
        tx: &'a Transaction,
        block_hash: H256,
    ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
        try_join(
            self.0.process_tx(tx, block_hash),
            self.1.process_tx(tx, block_hash),
        )
        .boxed()
    }
}

impl<E, M1, M2> BlockMonitor for (M1, M2)
where
    E: Send + 'static,
    M1: BlockMonitor<Error = E>,
    M1::Ok: Send,
    M2: BlockMonitor<Error = E>,
    M2::Ok: Send,
{
    type Ok = (M1::Ok, M2::Ok);

    type Error = E;

    fn process_block<'a>(
        &'a self,
        block: &'a Block<Transaction>,
    ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
        try_join(self.0.process_block(block), self.1.process_block(block)).boxed()
    }
}
