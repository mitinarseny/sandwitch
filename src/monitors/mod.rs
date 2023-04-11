use core::convert::Infallible;
use std::{
    cmp::Reverse,
    collections::HashMap,
    iter::Map,
    marker::PhantomData,
    ops::{Add, AddAssign, Deref, DerefMut, Index},
    slice::SliceIndex,
    sync::Arc,
};

use contracts::multicall::{Call, Calls, MultiCall, RawCall, TryCall};
use ethers::types::{Block, Log, Transaction, TxHash, U256};
use futures::{
    future::{self, BoxFuture, FutureExt, TryFutureExt},
    lock::Mutex,
    stream::{FuturesUnordered, TryStreamExt},
};
use impl_tools::autoimpl;
use itertools::Itertools;
use tracing::{event, Level};

#[derive(Default)]
#[autoimpl(Deref<Target = [TryCall<RawCall>]> using self.0)]
pub struct MultiCallGroups(Calls<RawCall>);

impl<C: Call> Extend<C> for MultiCallGroups {
    fn extend<T: IntoIterator<Item = C>>(&mut self, calls: T) {
        self.0.extend(calls.into_iter().map(|c| {
            let (call, _meta) = c.encode_raw();
            TryCall {
                allow_failure: true,
                call,
            }
        }))
    }
}

impl<C: Call> FromIterator<C> for MultiCallGroups {
    fn from_iter<T: IntoIterator<Item = C>>(calls: T) -> Self {
        let mut this = Self::default();
        this.extend(calls);
        this
    }
}

impl IntoIterator for MultiCallGroups {
    type Item = RawCall;
    type IntoIter =
        Map<<Calls<RawCall> as IntoIterator>::IntoIter, fn(TryCall<RawCall>) -> RawCall>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().map(TryCall::into_call)
    }
}

#[autoimpl(Deref using self.calls)]
#[autoimpl(DerefMut using self.calls)]
pub struct PrioritizedMultiCall {
    pub calls: MultiCallGroups, // TODO: no pub
    pub priority_fee: U256,     // TODO: no pub
}

impl PrioritizedMultiCall {
    pub fn new(calls: MultiCallGroups, priority_fee: impl Into<U256>) -> Self {
        // TODO: no pub
        Self {
            calls,
            priority_fee: priority_fee.into(),
        }
    }
}

#[autoimpl(Deref using self.tx)]
pub struct TxWithLogs {
    pub tx: Transaction,
    pub logs: Vec<Log>,
}

#[derive(Clone, Copy)]
#[autoimpl(Deref<Target = [TxWithLogs]> using self.txs)]
pub struct ContiniousPendingTxsWithLogs<'a> {
    txs: &'a [TxWithLogs],
    to_send: &'a ToSend,
}

impl<'a> ContiniousPendingTxsWithLogs<'a> {
    #[inline]
    pub fn priority_fee(&self) -> U256 {
        self.txs
            .first()
            .and_then(|tx| tx.gas_price)
            .expect("empty continuious transactions")
    }

    #[inline]
    pub fn before_priority_fee(&self) -> U256 {
        self.priority_fee() + 1
    }

    #[inline]
    pub fn make_before(&self, calls: MultiCallGroups) -> PrioritizedMultiCall {
        PrioritizedMultiCall::new(calls, self.before_priority_fee())
    }

    #[inline]
    pub fn make_after(&self, calls: MultiCallGroups) -> PrioritizedMultiCall {
        PrioritizedMultiCall::new(calls, self.priority_fee())
    }

    pub async fn add_to_send_before(&self, calls: MultiCallGroups) {
        self.to_send
            .add_to_send(Some(self.make_before(calls)))
            .await
    }

    pub async fn add_to_send_after(&self, calls: MultiCallGroups) {
        self.to_send.add_to_send(Some(self.make_after(calls))).await
    }

    pub async fn add_to_send_wrap(&self, before: MultiCallGroups, after: MultiCallGroups) {
        self.to_send
            .add_to_send([self.make_before(before), self.make_after(after)])
            .await
    }
}

#[derive(Default)]
pub struct ToSend(Mutex<Vec<PrioritizedMultiCall>>);

impl ToSend {
    pub async fn add_to_send(&self, calls: impl IntoIterator<Item = PrioritizedMultiCall>) {
        self.0.lock().await.extend(calls)
    }

    pub fn into_inner(self) -> Vec<PrioritizedMultiCall> {
        self.0.into_inner()
    }
}

#[autoimpl(Deref using self.block)]
pub struct PendingBlock {
    block: Block<TxWithLogs>,
    first_priority_fee: U256,
    to_send: ToSend,
}

impl PendingBlock {
    pub fn new(
        block: Block<Transaction>,
        logs: impl IntoIterator<Item = Log>,
        first_prioroty_fee: impl Into<U256>,
    ) -> Self {
        let Block {
            hash,
            parent_hash,
            uncles_hash,
            author,
            state_root,
            transactions_root,
            receipts_root,
            number,
            gas_used,
            gas_limit,
            extra_data,
            logs_bloom,
            timestamp,
            difficulty,
            total_difficulty,
            seal_fields,
            uncles,
            transactions,
            size,
            mix_hash,
            nonce,
            base_fee_per_gas,
            other,
        } = block;
        Self {
            block: Block {
                hash,
                parent_hash,
                uncles_hash,
                author,
                state_root,
                transactions_root,
                receipts_root,
                number,
                gas_used,
                gas_limit,
                extra_data,
                logs_bloom,
                timestamp,
                difficulty,
                total_difficulty,
                seal_fields,
                uncles,
                transactions: transactions
                    .into_iter()
                    .map({
                        let mut logs = logs.into_iter().peekable();
                        move |tx| TxWithLogs {
                            logs: logs
                                .peeking_take_while(|l| {
                                    l.transaction_hash.is_some_and(|tx_hash| tx_hash == tx.hash)
                                })
                                .collect(),
                            tx,
                        }
                    })
                    .collect(),
                size,
                mix_hash,
                nonce,
                base_fee_per_gas,
                other,
            },
            first_priority_fee: first_prioroty_fee.into(),
            to_send: Default::default(),
        }
    }

    pub fn iter_continious_txs(&self) -> impl Iterator<Item = ContiniousPendingTxsWithLogs<'_>> {
        self.block
            .transactions
            .group_by(|l, r| l.gas_price.unwrap() == r.gas_price.unwrap())
            .map(|txs| ContiniousPendingTxsWithLogs {
                txs,
                to_send: &self.to_send,
            })
    }

    #[inline]
    pub fn first_priority_fee(&self) -> U256 {
        self.first_priority_fee
    }

    fn make_first(&self, calls: MultiCallGroups) -> PrioritizedMultiCall {
        PrioritizedMultiCall::new(calls, self.first_priority_fee())
    }

    pub async fn add_to_send_first_in_block(&self, calls: MultiCallGroups) {
        self.to_send.add_to_send(Some(self.make_first(calls))).await
    }

    pub async fn add_to_send_first_in_block_and_wraps(
        &self,
        first: MultiCallGroups,
        wraps: impl IntoIterator<Item = PrioritizedMultiCall>,
    ) {
        self.to_send
            .add_to_send(Some(self.make_first(first)).into_iter().chain(wraps))
            .await
    }

    pub fn into_send_txs(self) -> Vec<PrioritizedMultiCall> {
        self.to_send.into_inner()
    }
}

#[autoimpl(for<M: trait + ?Sized> Arc<M>, Box<M>)]
pub trait PendingBlockMonitor {
    fn process_pending_block<'a>(
        &'a self,
        block: &'a PendingBlock,
    ) -> BoxFuture<'a, anyhow::Result<()>>;
}

impl PendingBlockMonitor for () {
    fn process_pending_block<'a>(
        &'a self,
        _block: &'a PendingBlock,
    ) -> BoxFuture<'a, anyhow::Result<()>> {
        future::ok(()).boxed()
    }
}

#[autoimpl(Deref using self.0)]
#[autoimpl(DerefMut using self.0)]
pub struct MultiMonitor<M>(Vec<M>);

impl<M> From<Vec<M>> for MultiMonitor<M> {
    fn from(monitors: Vec<M>) -> Self {
        Self(monitors)
    }
}

impl<M> MultiMonitor<M> {
    pub fn into_inner(self) -> Vec<M> {
        self.0
    }
}

impl<M: PendingBlockMonitor> PendingBlockMonitor for MultiMonitor<M> {
    fn process_pending_block<'a>(
        &'a self,
        block: &'a PendingBlock,
    ) -> BoxFuture<'a, anyhow::Result<()>> {
        self.0
            .iter()
            .map(|m| m.process_pending_block(block))
            .collect::<FuturesUnordered<_>>()
            .try_collect()
            .boxed()
    }
}

pub struct TxMonitor<M>(M);

impl<M: PendingTxMonitor> PendingBlockMonitor for TxMonitor<M> {
    fn process_pending_block<'a>(
        &'a self,
        block: &'a PendingBlock,
    ) -> BoxFuture<'a, anyhow::Result<()>> {
        block
            .transactions
            .iter()
            .map(|tx| self.0.process_pending_tx(tx))
            .collect::<FuturesUnordered<_>>()
            .try_collect()
            .boxed()
    }
}

pub trait PendingTxMonitor {
    fn process_pending_tx<'a>(&'a self, tx: &'a Transaction) -> BoxFuture<'a, anyhow::Result<()>>;
}

impl PendingTxMonitor for () {
    fn process_pending_tx<'a>(&'a self, _tx: &'a Transaction) -> BoxFuture<'a, anyhow::Result<()>> {
        future::ok(()).boxed()
    }
}

impl<M: PendingTxMonitor> PendingTxMonitor for MultiMonitor<M> {
    fn process_pending_tx<'a>(&'a self, tx: &'a Transaction) -> BoxFuture<'a, anyhow::Result<()>> {
        self.0
            .iter()
            .map(|m| m.process_pending_tx(tx))
            .collect::<FuturesUnordered<_>>()
            .try_collect()
            .boxed()
    }
}

pub trait ThinSandwichMonitor {
    fn wrap_continious_pending_txs<'a>(
        &'a self,
        txs: &'a [TxWithLogs],
    ) -> BoxFuture<'a, anyhow::Result<[MultiCallGroups; 2]>>;
}

pub struct ThinSandwichWrapper<M>(M);

impl<M> PendingBlockMonitor for ThinSandwichWrapper<M>
where
    M: ThinSandwichMonitor + Send + Sync,
{
    fn process_pending_block<'a>(
        &'a self,
        block: &'a PendingBlock,
    ) -> BoxFuture<'a, anyhow::Result<()>> {
        block
            .iter_continious_txs()
            .map(|txs| async move {
                let [before, after] = self.0.wrap_continious_pending_txs(&txs).await?;
                txs.add_to_send_wrap(before, after).await;
                Ok(())
            })
            .collect::<FuturesUnordered<_>>()
            .try_collect()
            .boxed()
    }
}

// TODO: contract call monitor, function call monitor

// // pub mod erc20_utils;
// // pub mod inputs;

// // #[cfg(feature = "pancake_swap")]
// // pub mod pancake_swap;

// // pub(crate) trait BlockMonitor {
// //     type Error;
// //
// //     fn process_block<'a>(
// //         &'a mut self,
// //         block: &'a Block<Transaction>,
// //     ) -> BoxFuture<'a, Result<(), Self::Error>>;
// // }

// pub struct MultiCallRequest {
//     value: U256,
//     calls: Calls<RawCall>,
// }

// struct MultiCallRequests {
//     first_in_block: Calls<RawCall>,
// }

// // #[derive(Default)]
// // pub(crate) struct ProcessState {
// //     send_txs: Mutex<Vec<TransactionRequest>>,
// // }

// // impl ProcessState {
// //     pub(crate) async fn send_txs(&self, txs: impl IntoIterator<Item = TransactionRequest>) {
// //         let mut send_txs = self.send_txs.lock().await;
// //         send_txs.extend(txs);
// //     }

// //     pub(crate) fn into_txs(self) -> Vec<TransactionRequest> {
// //         let mut txs = self.send_txs.into_inner();
// //         txs.sort_by
// //     }
// // }

// #[autoimpl(Deref using self.tx)]
// #[derive(Default, Debug)]
// pub(crate) struct TransactionWithLogs {
//     pub tx: Transaction,
//     pub logs: Vec<Log>,
// }

// #[autoimpl(Deref using self.inner)]
// struct PendingTransactionWithLog<'a> {
//     inner: &'a TransactionWithLogs,
//     to_send: &'a TransactionRequests,
// }

// impl<'a> PendingTransactionWithLog<'a> {
//     async fn front_run(
//         &self,
//         txs: impl IntoIterator<Item = TransactionRequest>,
//         first_in_block: bool,
//     ) {
//     }

//     async fn back_run(&self, txs: impl IntoIterator<Item = TransactionRequest>) {}

//     async fn front_back_run(
//         &self,
//         front: impl IntoIterator<Item = TransactionRequest>,
//         back: impl IntoIterator<Item = TransactionRequest>,
//         first_in_block: bool,
//     ) {
//     }
// }

// pub(crate) struct MapErr<M, F> {
//     inner: M,
//     f: F,
// }
//
// pub(crate) struct ErrInto<M, E> {
//     inner: M,
//     _into: PhantomData<E>,
// }
//
// impl<M, F, E> PendingBlockMonitor for MapErr<M, F>
// where
//     M: PendingBlockMonitor,
//     F: Fn(M::Error) -> E + Sync,
//     E: 'static,
// {
//     type Error = E;
//
//     fn process_pending_block<'a>(
//         &'a self,
//         block: &'a Block<TransactionWithLogs>,
//     ) -> BoxFuture<'a, Result<(), Self::Error>> {
//         self.inner
//             .process_pending_block(block)
//             .map_err(&self.f)
//             .boxed()
//     }
// }
//
// impl<M, E> PendingBlockMonitor for ErrInto<M, E>
// where
//     M: PendingBlockMonitor,
//     M::Error: Into<E>,
//     E: 'static,
// {
//     type Error = E;
//
//     fn process_pending_block<'a>(
//         &'a self,
//         block: &'a Block<TransactionWithLogs>,
//     ) -> BoxFuture<'a, Result<(), Self::Error>> {
//         self.inner.process_pending_block(block).err_into().boxed()
//     }
// }
//
// pub(crate) trait PendingBlockMonitorExt: PendingBlockMonitor {
//     fn map_err<F, E>(self, f: F) -> MapErr<Self, F>
//     where
//         Self: Sized,
//         F: Fn(Self::Error) -> E + Sync,
//         E: 'static,
//     {
//         MapErr { inner: self, f }
//     }
//
//     fn err_into<E>(self) -> ErrInto<Self, E>
//     where
//         Self: Sized,
//         Self::Error: Into<E>,
//         E: 'static,
//     {
//         ErrInto {
//             inner: self,
//             _into: PhantomData,
//         }
//     }
// }

// trait Monitor {
//     type Args;
//     type Error: 'static;
//
//     fn process<'a>(&'a self, args: Self::Args) -> BoxFuture<'a, Result<(), Self::Error>>;
// }
//
// impl<'a, M> PendingBlockMonitorExt for M where M: PendingBlockMonitor {}
//
// trait MultiPendingBlockMonitor<M>
// where
//     for<'a> &'a Self: IntoIterator<Item = &'a M>,
//     M: PendingBlockMonitor,
// {
//     fn into_multi_monitor(self) -> MultiMonitor<Self> {
//         MultiMonitor(self)
//     }
// }
//
// impl<I, M> MultiPendingBlockMonitor<M> for I
// where
//     for<'a> &'a Self: IntoIterator<Item = &'a M>,
//     M: PendingBlockMonitor,
// {
// }
//
// pub(crate) struct MultiMonitor<I>(I)
// where
//     for<'a> &'a I: IntoIterator;
//
// impl<I> From<I> for MultiMonitor<I>
// where
//     for<'a> &'a I: IntoIterator,
// {
//     fn from(value: I) -> Self {
//         Self(value)
//     }
// }
//
// impl<I, M> PendingBlockMonitor for MultiMonitor<I>
// where
//     for<'a> &'a I: IntoIterator<Item = &'a M>,
//     M: PendingBlockMonitor,
// {
//     type Error = M::Error;
//
//     fn process_pending_block<'a>(
//         &'a self,
//         block: &'a Block<TransactionWithLogs>,
//     ) -> BoxFuture<'a, Result<(), Self::Error>> {
//         self.0
//             .into_iter()
//             .map(|m| m.process_pending_block(block))
//             .collect::<FuturesUnordered<_>>()
//             .try_collect()
//             .boxed()
//     }
// }

// impl<M> PendingBlockMonitor for [M]
// where
//     M: PendingBlockMonitor,
// {
//     type Error = M::Error;
//
//     fn process_pending_block<'a>(
//         &'a self,
//         block: &'a Block<TransactionWithLogs>,
//     ) -> BoxFuture<'a, Result<(), Self::Error>> {
//         self.iter()
//             .map(|m| m.process_pending_block(block))
//             .collect::<FuturesUnordered<_>>()
//             .try_collect()
//             .boxed()
//     }
// }
//
// impl<M, const N: usize> PendingBlockMonitor for [M; N]
// where
//     M: PendingBlockMonitor,
// {
//     type Error;
//
//     fn process_pending_block<'a>(
//         &'a self,
//         block: &'a Block<TransactionWithLogs>,
//     ) -> BoxFuture<'a, Result<(), Self::Error>> {
//         self.as_slice().process_pending_block(block)
//     }
// }
//
// impl<M> PendingBlockMonitor for

// impl<M> PendingBlockMonitor for HashMap<K, M>
// where
//     M: PendingBlockMonitor,
//     M::Error: Into<anyhow::Error>,
// {
//     type Error = anyhow::Error;
//
//     fn process_pending_block<'a>(
//         &'a self,
//         block: &'a Block<TransactionWithLogs>,
//     ) -> BoxFuture<'a, Result<(), Self::Error>> {
//         self.iter()
//             .map(|(name, m)| m.process_pending_block(block).map(|r| r.with_context()))
//             .collect::<FuturesUnordered<_>>()
//             .try_collect()
//             .boxed()
//     }
// }

// pub(crate) struct Noop;

// impl PendingBlockMonitor for Noop {
//     type Error = Infallible;

//     fn process_pending_block<'a>(
//         &'a self,
//         _block: &'a Block<TransactionWithLogs>,
//     ) -> BoxFuture<'a, Result<(), Self::Error>> {
//         future::ok(()).boxed()
//     }
// }

// pub(crate) trait State: Default + Sync + Send {}

// impl State for () {}

// pub(crate) trait FrontRunMonitor: Sync {
//     type Error: 'static;
//     type State: State;

//     // TODO: return bool to indicate if we should continue on next txs
//     fn process_pending_tx<'a>(
//         &'a self,
//         tx: &'a TransactionWithLogs,
//         state: &'a Self::State,
//     ) -> BoxFuture<'a, Result<(), Self::Error>>;
// }

// impl FrontRunMonitor for Noop {
//     type Error = Infallible;

//     type State = ();

//     fn process_pending_tx<'a>(
//         &'a self,
//         tx: &'a TransactionWithLogs,
//         state: &'a Self::State,
//     ) -> BoxFuture<'a, Result<(), Self::Error>> {
//         future::ok(()).boxed()
//     }
// }

// pub(crate) struct FrontRun<M: FrontRunMonitor> {
//     inner: M,
// }

// impl<M> PendingBlockMonitor for FrontRun<M>
// where
//     M: FrontRunMonitor,
// {
//     type Error = M::Error;

//     fn process_pending_block<'a>(
//         &'a self,
//         block: &'a Block<TransactionWithLogs>,
//     ) -> BoxFuture<'a, Result<(), Self::Error>> {
//         let state = <M::State>::default();
//         async move {
//             block
//                 .transactions
//                 .iter()
//                 .map(|tx| self.inner.process_pending_tx(tx, &state))
//                 .collect::<FuturesUnordered<_>>()
//                 .try_collect()
//                 .await
//         }
//         .boxed()
//     }
// }

// struct A<M: PendingBlockMonitor> {
//     monitor: M,
// }

// impl<M: PendingBlockMonitor> A<M> {}

// fn f() {
//     let a: A<MultiMonitor<Vec<Box<dyn PendingBlockMonitor<Error = anyhow::Error>>>>> = A {
//         monitor: vec![Box::new(FrontRun { inner: Noop }.err_into())
//             as Box<dyn PendingBlockMonitor<Error = anyhow::Error>>]
//         .into(),
//     };
//     a.monitor.process_pending_block(&Block::default());
// }

//
// impl<I, M> BlockMonitor for MultiMonitor<I>
// where
//     for<'a> &'a mut I: IntoIterator<Item = &'a mut M>,
//     M: BlockMonitor,
// {
//     type Error = M::Error;
//
//     fn process_block<'a>(
//         &'a mut self,
//         block: &'a Block<Transaction>,
//     ) -> BoxFuture<'a, Result<(), Self::Error>> {
//         self.0
//             .into_iter()
//             .map(|m| m.process_block(block))
//             .collect::<FuturesUnordered<_>>()
//             .try_collect::<()>()
//             .boxed()
//     }
// }
//
// pub(crate) trait BlockMonitorExt: BlockMonitor {
//     fn map_err<F, E>(self, f: F) -> MapErr<Self, F>
//     where
//         Self: Sized,
//         F: Fn(Self::Error) -> E + Sync,
//     {
//         MapErr { inner: self, f }
//     }
//
//     fn err_into<E>(self) -> ErrInto<Self, E>
//     where
//         Self: Sized,
//         Self::Error: Into<E>,
//     {
//         ErrInto {
//             inner: self,
//             _into: PhantomData,
//         }
//     }
// }
//
// impl<M> BlockMonitorExt for M where M: BlockMonitor {}
//
// pub(crate) struct Noop<E: Send>(PhantomData<E>);
//
// impl<E: Send> Default for Noop<E> {
//     fn default() -> Self {
//         Self(PhantomData)
//     }
// }
//
// impl<E: Send> BlockMonitor for Noop<E> {
//     type Ok = ();
//     type Error = E;
//
//     fn process_block<'a>(
//         &'a self,
//         _block: &'a Block<Transaction>,
//     ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
//         future::ok(()).boxed()
//     }
// }
//
// impl<E: Send> PendingBlockMonitor for Noop<E> {
//     type Error = E;
//
//     fn process_pending_block<'a>(
//         &'a mut self,
//         block: &'a Block<Transaction>,
//     ) -> BoxFuture<'a, Result<(), Self::Error>> {
//         future::ok(()).boxed()
//     }
// }
//
// impl<M: ?Sized> BlockMonitor for Box<M>
// where
//     M: BlockMonitor,
// {
//     type Error = M::Error;
//
//     fn process_block<'a>(
//         &'a mut self,
//         block: &'a Block<Transaction>,
//     ) -> BoxFuture<'a, Result<(), Self::Error>> {
//         (**self).process_block(block)
//     }
// }
//
// impl<M: ?Sized> PendingBlockMonitor for Box<M>
// where
//     M: PendingBlockMonitor,
// {
//     type Error = M::Error;
//
//     fn process_pending_block<'a>(
//         &'a mut self,
//         block: &'a Block<Transaction>,
//     ) -> BoxFuture<'a, Result<(), Self::Error>> {
//         (**self).process_pending_block(block)
//     }
// }
//
// impl<M: ?Sized> BlockMonitor for Arc<M>
// where
//     M: BlockMonitor,
// {
//     type Error = M::Error;
//
//     fn process_block<'a>(
//         &'a mut self,
//         block: &'a Block<Transaction>,
//     ) -> BoxFuture<'a, Result<(), Self::Error>> {
//         (**self).process_block(block)
//     }
// }
//
// impl<M: ?Sized> PendingBlockMonitor for Arc<M>
// where
//     M: PendingBlockMonitor,
// {
//     type Error = M::Error;
//
//     fn process_pending_block<'a>(
//         &'a mut self,
//         block: &'a Block<Transaction>,
//     ) -> BoxFuture<'a, Result<(), Self::Error>> {
//         (**self).process_pending_block(block)
//     }
// }
//
// impl<M> BlockMonitor for [M]
// where
//     M: BlockMonitor,
//     M::Error: Send,
// {
//     type Error = M::Error;
//
//     fn process_block<'a>(
//         &'a mut self,
//         block: &'a Block<Transaction>,
//     ) -> BoxFuture<'a, Result<(), Self::Error>> {
//         self.iter_mut()
//             .map(|m| m.process_block(block))
//             .collect::<FuturesUnordered<_>>()
//             .try_collect::<()>()
//             .boxed()
//     }
// }
//
// impl<M> PendingBlockMonitor for [M]
// where
//     M: PendingBlockMonitor,
//     M::Error: Send,
// {
//     type Error = M::Error;
//
//     fn process_pending_block<'a>(
//         &'a mut self,
//         block: &'a Block<Transaction>,
//     ) -> BoxFuture<'a, Result<(), Self::Error>> {
//         self.iter_mut()
//             .map(|m| m.process_pending_block(block))
//             .collect::<FuturesUnordered<_>>()
//             .try_collect::<()>()
//             .boxed()
//     }
// }
//
// impl<M> BlockMonitor for Vec<M>
// where
//     M: BlockMonitor,
//     M::Error: Send,
// {
//     type Error = M::Error;
//
//     fn process_block<'a>(
//         &'a self,
//         block: &'a Block<Transaction>,
//     ) -> BoxFuture<'a, Result<(), Self::Error>> {
//         (**self).process_block(block)
//     }
// }
//
// pub(crate) trait FilterTxMonitor {
//     type Ok;
//     type Error;
//
//     fn filter(&self, tx: &Transaction) -> bool {
//         true
//     }
//
//     fn process_filtered_tx<'a>(
//         &'a self,
//         tx: &'a Transaction,
//         block_hash: H256,
//     ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>>;
// }
//
// pub(crate) trait FilterTxMonitorExt: FilterTxMonitor {
//     fn maybe_process_tx<'a>(
//         &'a self,
//         tx: &'a Transaction,
//         block_hash: H256,
//     ) -> Option<BoxFuture<'a, Result<Self::Ok, Self::Error>>> {
//         if !self.filter(tx) {
//             return None;
//         }
//         Some(self.process_filtered_tx(tx, block_hash))
//     }
// }
//
// impl<M> FilterTxMonitorExt for M where M: FilterTxMonitor {}
//
// pub(crate) trait ContractCallMonitor<'a, C: 'a> {
//     type Ok;
//     type Error;
//
//     fn filter(&self, tx_to: Address) -> bool {
//         true
//     }
//
//     fn process_call(
//         &'a self,
//         tx: &'a Transaction,
//         block_hash: H256,
//         inputs: C,
//     ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>>;
// }
//
// pub(crate) trait ContractCallMonitorExt<'a, C: AbiDecode + 'a>:
//     ContractCallMonitor<'a, C>
// {
//     fn maybe_process_call(
//         &'a self,
//         tx: &'a Transaction,
//         block_hash: H256,
//     ) -> Option<BoxFuture<'a, Result<Self::Ok, Self::Error>>> {
//         if !self.filter(tx.to?) {
//             return None;
//         }
//         Some(self.process_call(tx, block_hash, C::decode(&tx.input).ok()?))
//     }
// }
//
// impl<'a, C: AbiDecode + 'a, M: ContractCallMonitor<'a, C>> ContractCallMonitorExt<'a, C> for M {}
//
// pub(crate) trait FunctionCallMonitor<'a, C: EthCall + 'a> {
//     type Ok;
//     type Error;
//
//     fn process_func(
//         &'a self,
//         tx: &'a Transaction,
//         block_hash: H256,
//         inputs: C,
//     ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>>;
// }
//
// pub(crate) struct MapErr<M, F> {
//     inner: M,
//     f: F,
// }
//
// impl<M, F, E> BlockMonitor for MapErr<M, F>
// where
//     M: BlockMonitor,
//     F: Fn(M::Error) -> E + Sync,
// {
//     type Error = E;
//
//     fn process_block<'a>(
//         &'a mut self,
//         block: &'a Block<Transaction>,
//     ) -> BoxFuture<'a, Result<(), Self::Error>> {
//         self.inner.process_block(block).map_err(&self.f).boxed()
//     }
// }
//
// impl<M, F, E> PendingBlockMonitor for MapErr<M, F>
// where
//     M: PendingBlockMonitor,
//     F: Fn(M::Error) -> E + Sync,
// {
//     type Error = E;
//
//     fn process_pending_block<'a>(
//         &'a mut self,
//         block: &'a Block<Transaction>,
//     ) -> BoxFuture<'a, Result<(), Self::Error>> {
//         self.inner
//             .process_pending_block(block)
//             .map_err(&self.f)
//             .boxed()
//     }
// }
//
// impl<'a, M, C, F, E> FunctionCallMonitor<'a, C> for MapErr<M, F>
// where
//     C: EthCall + 'a,
//     M: FunctionCallMonitor<'a, C>,
//     F: Fn(M::Error) -> E + Sync,
// {
//     type Ok = M::Ok;
//     type Error = E;
//
//     fn process_func(
//         &'a self,
//         tx: &'a Transaction,
//         block_hash: H256,
//         inputs: C,
//     ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
//         self.inner
//             .process_func(tx, block_hash, inputs)
//             .map_err(&self.f)
//             .boxed()
//     }
// }
//
// pub(crate) struct ErrInto<M, E> {
//     inner: M,
//     _into: PhantomData<E>,
// }
//
// impl<M, E> BlockMonitor for ErrInto<M, E>
// where
//     M: BlockMonitor,
//     M::Error: Into<E>,
// {
//     type Ok = M::Ok;
//     type Error = E;
//
//     fn process_block<'a>(
//         &'a mut self,
//         block: &'a Block<Transaction>,
//     ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
//         self.inner.process_block(block).err_into().boxed()
//     }
// }
//
// impl<M, E> PendingBlockMonitor for ErrInto<M, E>
// where
//     M: PendingBlockMonitor,
//     M::Error: Into<E>,
// {
//     type Ok = M::Ok;
//     type Error = E;
//
//     fn process_pending_block<'a>(
//         &'a mut self,
//         block: &'a Block<Transaction>,
//     ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
//         self.inner.process_pending_block(block).err_into().boxed()
//     }
// }
//
// impl<'a, M, C, E> FunctionCallMonitor<'a, C> for ErrInto<M, E>
// where
//     C: EthCall + 'a,
//     M: FunctionCallMonitor<'a, C>,
//     M::Error: Into<E>,
// {
//     type Ok = M::Ok;
//     type Error = E;
//
//     fn process_func(
//         &'a self,
//         tx: &'a Transaction,
//         block_hash: H256,
//         inputs: C,
//     ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
//         self.inner
//             .process_func(tx, block_hash, inputs)
//             .err_into()
//             .boxed()
//     }
// }
//
// impl<E, M1, M2> TxMonitor for (M1, M2)
// where
//     E: Send + 'static,
//     M1: TxMonitor<Error = E>,
//     M1::Ok: Send,
//     M2: TxMonitor<Error = E>,
//     M2::Ok: Send,
// {
//     type Ok = (M1::Ok, M2::Ok);
//
//     type Error = E;
//
//     fn process_tx<'a>(
//         &'a self,
//         tx: &'a Transaction,
//         block_hash: H256,
//     ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
//         try_join(
//             self.0.process_tx(tx, block_hash),
//             self.1.process_tx(tx, block_hash),
//         )
//         .boxed()
//     }
// }
//
// impl<E, M1, M2> BlockMonitor for (M1, M2)
// where
//     E: Send + 'static,
//     M1: BlockMonitor<Error = E>,
//     M1::Ok: Send,
//     M2: BlockMonitor<Error = E>,
//     M2::Ok: Send,
// {
//     type Ok = (M1::Ok, M2::Ok);
//
//     type Error = E;
//
//     fn process_block<'a>(
//         &'a self,
//         block: &'a Block<Transaction>,
//     ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
//         try_join(self.0.process_block(block), self.1.process_block(block)).boxed()
//     }
// }
