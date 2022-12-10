use std::marker::PhantomData;

use ethers::{
    contract::EthCall,
    types::{Transaction, H256},
};
use futures::{future::BoxFuture, stream::FuturesUnordered, FutureExt, TryFutureExt, TryStreamExt};

#[cfg(feature = "pancake_swap")]
pub mod pancake_swap;

pub trait TxMonitor {
    type Ok;
    type Error;

    fn process_tx<'a>(
        &'a self,
        tx: &'a Transaction,
        block_hash: H256,
    ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>>;
}

pub trait TxMonitorExt: TxMonitor {
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

impl<M> TxMonitor for [M]
where
    M: TxMonitor,
    M::Ok: Send,
{
    type Ok = Vec<M::Ok>;
    type Error = M::Error;

    fn process_tx<'a>(
        &'a self,
        tx: &'a Transaction,
        block_hash: H256,
    ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
        self.iter()
            .map(|m| m.process_tx(tx, block_hash))
            .collect::<FuturesUnordered<_>>()
            .try_collect()
            .boxed()
    }
}

impl<M> TxMonitor for Vec<M>
where
    M: TxMonitor,
    M::Ok: Send,
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

pub trait FunctionCallMonitor<'a, C: EthCall + 'a> {
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

pub struct MapErr<M, F> {
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

pub struct ErrInto<M, E> {
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
