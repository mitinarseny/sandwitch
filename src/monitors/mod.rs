use std::{marker::PhantomData, sync::Arc};

use ethers::{
    contract::EthCall,
    providers::JsonRpcClient,
    signers::Signer,
    types::{Transaction, H256},
};
use futures::{future::BoxFuture, stream::FuturesUnordered, FutureExt, TryFutureExt, TryStreamExt};

use crate::accounts::Accounts;

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

// impl<M: ?Sized + Sync + Send> TxMonitor for Arc<M>
// where
//     M: TxMonitor,
// {
//     type Ok = M::Ok;
//     type Error = M::Error;
//
//     fn process_tx<'a, P: JsonRpcClient, S: Signer>(
//         &'a self,
//         tx: &'a Transaction, // TODO: arc?
//         block_hash: H256,
//         accounts: &'a Accounts<P, S>,
//     ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
//         (**self).process_tx(tx, block_hash, accounts)
//     }
// }

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

// pub trait TryFunctionCallMonitor<'a, C: EthCall>: 'a {
//     type Ok;
//     type Error;
//
//     fn try_process_func<P: JsonRpcClient, S: Signer>(
//         self,
//         tx: &'a Transaction,
//         block_hash: H256,
//         inputs: C,
//         accounts: &'a Accounts<P, S>,
//     ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>>;
// }
//
// impl<'a, M, C, T, E> TryFunctionCallMonitor<'a, C> for M
// where
//     C: EthCall,
//     M: ?Sized + FunctionCallMonitor<'a, C, Output = Result<T, E>>,
// {
//     type Ok = T;
//     type Error = E;
//
//     fn try_process_func<P: JsonRpcClient, S: Signer>(
//         self,
//         tx: &'a Transaction,
//         block_hash: H256,
//         inputs: C,
//         accounts: &'a Accounts<P, S>,
//     ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
//         self.process_func(tx, block_hash, inputs, accounts)
//     }
// }

// pub struct NeverError<'a, T: 'a> {
//     inner: T,
// }
//
// impl<'a, M> TxMonitor<'a> for NeverError<'a, M>
// where
//     M: TxMonitor<'a>,
// {
//     type Output = Result<M::Output, Infallible>;
//
//     fn process_tx<P: JsonRpcClient, S: Signer>(
//         self,
//         tx: &'a Transaction,
//         block_hash: H256,
//         accounts: &'a Accounts<P, S>,
//     ) -> BoxFuture<'a, Self::Output> {
//         self.inner
//             .process_tx(tx, block_hash, accounts)
//             .never_error()
//             .boxed()
//     }
// }
//
// impl<'a, M, C> FunctionCallMonitor<'a, C> for NeverError<'a, M>
// where
//     C: EthCall,
//     M: FunctionCallMonitor<'a, C>,
// {
//     type Output = Result<M::Output, Infallible>;
//
//     fn process_func<P: JsonRpcClient, S: Signer>(
//         self,
//         tx: &'a Transaction,
//         block_hash: H256,
//         inputs: C,
//         accounts: &'a Accounts<P, S>,
//     ) -> BoxFuture<'a, Self::Output> {
//         self.inner
//             .process_func(tx, block_hash, inputs, accounts)
//             .never_error()
//             .boxed()
//     }
// }
//
// pub struct ErrInto<'a, T: 'a, E> {
//     inner: T,
// }
//
// impl<'a, M, E> TxMonitor<'a> for ErrInto<'a, M, E>
// where
//     M: TryTxMonitor<'a>,
//     M::Error: Into<E>,
// {
//     type Output = Result<M::Ok, E>;
//
//     fn process_tx<P: JsonRpcClient, S: Signer>(
//         self,
//         tx: &'a Transaction,
//         block_hash: H256,
//         accounts: &'a Accounts<P, S>,
//     ) -> BoxFuture<'a, Self::Output> {
//         self.inner
//             .try_process_tx(tx, block_hash, accounts)
//             .err_into()
//             .boxed()
//     }
// }
//
// impl<'a, M, C, E> FunctionCallMonitor<'a, C> for ErrInto<'a, M, E>
// where
//     C: EthCall,
//     M: TryFunctionCallMonitor<'a, C>,
//     M::Error: Into<E>,
// {
//     type Output = Result<M::Ok, E>;
//
//     fn process_func<P: JsonRpcClient, S: Signer>(
//         self,
//         tx: &'a Transaction,
//         block_hash: H256,
//         inputs: C,
//         accounts: &'a Accounts<P, S>,
//     ) -> BoxFuture<'a, Self::Output> {
//         self.inner
//             .try_process_func(tx, block_hash, inputs, accounts)
//             .err_into()
//             .boxed()
//     }
// }

// impl<'a, M> Arc<M> where &'a M: TxMonitor<'a> {}
//
// impl<'a, M> TxMonitor<'a> for Arc<M>
// where
//     Input: 'a,
//     &'a M: TxMonitor<'a, &'a Input>,
// {
//     fn process_tx(
//         self,
//         input: Arc<Input>,
//         block_number: u64,
//         cancel: CancellationToken,
//     ) -> BoxFuture<'a, ()> {
//         async move { (*self).process_tx(&input, block_number, cancel) }.boxed()
//     }
// }
//
// pub trait BlockMonitor<'a>: 'a {
//     // fn process_block(self, block: );
// }

// pub trait TryMonitor<'a, Input: 'a>: 'a {
//     fn try_process(
//         self,
//         input: Input,
//         block_number: u64,
//         cancel: CancellationToken,
//     ) -> BoxFuture<'a, anyhow::Result<()>>;
// }
//
// impl<'a, I: 'a, M: ?Sized> TryMonitor<'a, I> for &'a Box<M>
// where
//     &'a M: TryMonitor<'a, I>,
// {
//     fn try_process(self, input: I, cancel: CancellationToken) -> BoxFuture<'a, anyhow::Result<()>> {
//         (**self).try_process(input, cancel)
//     }
// }
//
// impl<'a, I: 'a, M: ?Sized> TryMonitor<'a, I> for &'a mut Box<M>
// where
//     &'a mut M: TryMonitor<'a, I>,
// {
//     fn try_process(self, input: I, cancel: CancellationToken) -> BoxFuture<'a, anyhow::Result<()>> {
//         (**self).try_process(input, cancel)
//     }
// }
//
// impl<'a, I: 'a, M: ?Sized> TryMonitor<'a, I> for &'a Arc<M>
// where
//     &'a M: TryMonitor<'a, I>,
// {
//     fn try_process(self, input: I, cancel: CancellationToken) -> BoxFuture<'a, anyhow::Result<()>> {
//         (**self).try_process(input, cancel)
//     }
// }
//
// impl<'a, I: 'a, M: ?Sized> TryMonitor<'a, I> for &'a Mutex<M>
// where
//     I: Send,
//     M: Send,
//     &'a mut M: TryMonitor<'a, I>,
// {
//     fn try_process(self, input: I, cancel: CancellationToken) -> BoxFuture<'a, anyhow::Result<()>> {
//         async move { self.lock().await.try_process(input, cancel).await }.boxed()
//     }
// }
//
// impl<'a, I, M> TryMonitor<'a, I> for &'a [M]
// where
//     I: Send + 'a,
//     M: Send,
//     &'a M: TryMonitor<'a, I>,
// {
//     fn try_process(self, input: I, cancel: CancellationToken) -> BoxFuture<'a, anyhow::Result<()>> {
//         self.iter()
//             .map(|m| {
//                 tokio::spawn(TryMonitor::try_process(m, input, cancel.child_token()))
//                     .err_into()
//                     .map(Result::flatten)
//             })
//             .collect::<FuturesUnordered<_>>()
//             .try_collect::<()>()
//             .boxed()
//     }
// }
//
// impl<'a, I: 'a, M> TryMonitor<'a, I> for &'a mut [M]
// where
//     &'a mut M: TryMonitor<'a, I>,
// {
//     fn try_process(self, input: I, cancel: CancellationToken) -> BoxFuture<'a, anyhow::Result<()>> {
//         self.iter_mut()
//             .map(|m| {
//                 tokio::spawn(TryMonitor::try_process(m, input, cancel.child_token()))
//                     .err_into()
//                     .map(Result::flatten)
//             })
//             .collect::<FuturesUnordered<_>>()
//             .try_collect::<()>()
//             .boxed()
//     }
// }
//
// pub trait FunctionCallMonitor<'a, C: EthCall>: Sized + 'a {
//     fn on_func(
//         self,
//         tx: &'a Transaction,
//         inputs: C,
//         cancel: CancellationToken,
//     ) -> BoxFuture<'a, anyhow::Result<()>>;
//
//     fn on_func_raw(
//         self,
//         tx: &'a Transaction,
//         cancel: CancellationToken,
//     ) -> Option<BoxFuture<'a, anyhow::Result<()>>> {
//         let inputs = C::decode(&tx.input).ok()?;
//         Some(self.on_func(tx, inputs, cancel))
//     }
// }
//
// struct Noop();
//
// impl<'a> TryMonitor<'a, &'a Transaction> for &'a Noop {
//     fn try_process(
//         self,
//         input: &'a Transaction,
//         cancel: CancellationToken,
//     ) -> BoxFuture<'a, anyhow::Result<()>> {
//         todo!()
//     }
// }
//
// pub trait StatelessMonitor<'a, Input: 'a> {
//     fn process(
//         &'a self,
//         input: Input,
//         cancel: CancellationToken,
//     ) -> BoxFuture<'a, anyhow::Result<()>>;
// }
//
// impl<'a, I: 'a, M> StatelessMonitor<'a, I> for M
// where
//     &'a M: TryMonitor<'a, I>,
// {
//     fn process(&'a self, input: I, cancel: CancellationToken) -> BoxFuture<'a, anyhow::Result<()>> {
//         TryMonitor::try_process(self, input, cancel)
//     }
// }
//
// pub trait TxMonitor<'a>:
//     StatelessMonitor<'a, &'a Transaction> + StatelessMonitor<'a, &'a Block<TxHash>>
// {
// }
//
// impl<'a, M> TxMonitor<'a> for M where
//     M: StatelessMonitor<'a, &'a Transaction> + StatelessMonitor<'a, &'a Block<TxHash>>
// {
// }

// pub trait StatelessBlockMonitor: Send + Sync {
//     fn on_block<'a>(&'a self, block: &'a Block<TxHash>) -> BoxFuture<'a, anyhow::Result<()>>;
// }
//
// pub trait BlockMonitor: Send + Sync {
//     fn on_block<'a>(&'a mut self, block: &'a Block<TxHash>) -> BoxFuture<'a, anyhow::Result<()>>;
// }
//
// impl<M> StatelessBlockMonitor for Box<M>
// where
//     M: StatelessBlockMonitor + ?Sized,
// {
//     fn on_block<'a>(&'a self, block: &'a Block<TxHash>) -> BoxFuture<'a, anyhow::Result<()>> {
//         self.as_ref().on_block(block)
//     }
// }
//
// impl<M> StatelessBlockMonitor for RwLock<M>
// where
//     M: BlockMonitor,
// {
//     fn on_block<'a>(&'a self, block: &'a Block<TxHash>) -> BoxFuture<'a, anyhow::Result<()>> {
//         async move { self.write().await.on_block(block).await }.boxed()
//     }
// }
//
// impl<M> StatelessBlockMonitor for [M]
// where
//     M: StatelessBlockMonitor,
// {
//     fn on_block<'a>(&'a self, block: &'a Block<TxHash>) -> BoxFuture<'a, anyhow::Result<()>> {
//         self.iter()
//             .map(|m| m.on_block(block))
//             .collect::<FuturesUnordered<_>>()
//             .try_collect()
//             .boxed()
//     }
// }
//
// impl<M> BlockMonitor for Box<M>
// where
//     M: BlockMonitor + ?Sized,
// {
//     fn on_block<'a>(&'a mut self, block: &'a Block<TxHash>) -> BoxFuture<'a, anyhow::Result<()>> {
//         self.as_mut().on_block(block)
//     }
// }
//
// impl<M> BlockMonitor for RwLock<M>
// where
//     M: BlockMonitor,
// {
//     fn on_block<'a>(&'a mut self, block: &'a Block<TxHash>) -> BoxFuture<'a, anyhow::Result<()>> {
//         self.get_mut().on_block(block)
//     }
// }
//
// impl<M> BlockMonitor for [M]
// where
//     M: BlockMonitor,
// {
//     fn on_block<'a>(&'a mut self, block: &'a Block<TxHash>) -> BoxFuture<'a, anyhow::Result<()>> {
//         self.iter_mut()
//             .map(|m| m.on_block(block))
//             .collect::<FuturesUnordered<_>>()
//             .try_collect()
//             .boxed()
//     }
// }
//
// pub trait TxMonitor: Send + Sync {
//     /// NOTE: this is not cancel-safe
//     fn on_tx<'a>(
//         &'a self,
//         tx: &'a Transaction,
//         cancel: CancellationToken,
//     ) -> BoxFuture<'a, anyhow::Result<()>>;
// }
//
// impl<M> TxMonitor for Box<M>
// where
//     M: TxMonitor + ?Sized,
// {
//     fn on_tx<'a>(&'a self, tx: &'a Transaction) -> BoxFuture<'a, anyhow::Result<Vec<Bytes>>> {
//         (**self).on_tx(tx)
//     }
// }
//
// impl<M> TxMonitor for RwLock<M>
// where
//     M: TxMonitor,
// {
//     fn on_tx<'a>(&'a self, tx: &'a Transaction) -> BoxFuture<'a, anyhow::Result<Vec<Bytes>>> {
//         async move { self.read().await.on_tx(tx).await }.boxed()
//     }
// }
//
// impl<M> TxMonitor for [M]
// where
//     M: TxMonitor,
// {
//     fn on_tx<'a>(&'a self, tx: &'a Transaction) -> BoxFuture<'a, anyhow::Result<Vec<Bytes>>> {
//         self.iter()
//             .map(move |m| m.on_tx(tx))
//             .collect::<FuturesUnordered<_>>()
//             .try_concat()
//             .boxed()
//     }
// }
//
// pub trait Monitor: TxMonitor + StatelessBlockMonitor {}
// impl<M> Monitor for M where M: TxMonitor + StatelessBlockMonitor {}
//
// pub trait FunctionCallMonitor<C: EthCall>: Send + Sync {
//     fn on_func<'a>(
//         &'a self,
//         tx: &'a Transaction,
//         inputs: C,
//     ) -> BoxFuture<'a, anyhow::Result<Vec<Bytes>>>;
//
//     fn on_func_raw<'a>(
//         &'a self,
//         tx: &'a Transaction,
//     ) -> Option<BoxFuture<'a, anyhow::Result<Vec<Bytes>>>> {
//         let inputs = C::decode(&tx.input).ok()?;
//         Some(self.on_func(tx, inputs))
//     }
// }
