use core::{cmp::Reverse, iter::Map, mem};
use std::sync::Arc;

use contracts::multicall::{
    Call, Calls, MultiCall, MultiCallContract, MultiCallContractError, MultiFunctionCall, RawCall,
    TryCall,
};
use ethers::{
    providers::Middleware,
    types::{Address, Block, BlockNumber, Log, U256},
};
use futures::{
    future::{self, BoxFuture, FutureExt},
    lock::Mutex,
    stream::{FuturesUnordered, TryStreamExt},
};
use impl_tools::autoimpl;
use itertools::Itertools;

use crate::transactions::{InvalidTransaction, Transaction};

#[derive(Default, Debug)]
#[autoimpl(Deref<Target = [TryCall<RawCall>]> using self.calls)]
pub struct MultiCallGroups {
    calls: Calls<RawCall>,
    extended: bool,
}

impl MultiCallGroups {
    pub fn into_inner(self) -> Calls<RawCall> {
        self.calls
    }
}

impl<C: MultiCall> From<C> for MultiCallGroups {
    fn from(calls: C) -> Self {
        let (calls, _meta) = calls.encode_calls();
        Self {
            calls,
            extended: false,
        }
    }
}

impl<C: Call> Extend<C> for MultiCallGroups {
    fn extend<T: IntoIterator<Item = C>>(&mut self, calls: T) {
        if !self.extended {
            self.extended = true;
            let group = mem::take(&mut self.calls);
            self.extend(Some(group));
        }
        self.calls.extend(calls.into_iter().map(|c| {
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
        self.calls.into_iter().map(TryCall::into_call)
    }
}

#[autoimpl(Deref using self.calls)]
#[autoimpl(DerefMut using self.calls)]
pub struct PrioritizedMultiCall {
    pub calls: MultiCallGroups,     // TODO: no pub
    pub priority_fee_per_gas: U256, // TODO: no pub
}

impl PrioritizedMultiCall {
    fn new(calls: impl MultiCall, priority_fee_per_gas: impl Into<U256>) -> Self {
        Self {
            calls: calls.into(),
            priority_fee_per_gas: priority_fee_per_gas.into(),
        }
    }
}

#[derive(Debug)]
#[autoimpl(Deref using self.tx)]
pub struct TxWithLogs {
    pub tx: Transaction,
    pub logs: Vec<Log>,
}

#[derive(Clone, Copy)]
#[autoimpl(Deref<Target = [TxWithLogs]> using self.txs)]
pub struct AdjacentPendingTxsWithLogs<'a, M> {
    txs: &'a [TxWithLogs],
    block: &'a PendingBlock<'a, M>,
}

impl<'a, M> AdjacentPendingTxsWithLogs<'a, M> {
    fn priority_fee_per_gas(&self) -> U256 {
        self.txs
            .first()
            .map(|tx| self.block.base_fee_per_gas() + tx.fees.priority_fee())
            .expect("empty continuious transactions")
    }

    fn before_priority_fee_per_gas(&self) -> U256 {
        self.priority_fee_per_gas() + 1
    }

    pub fn make_before(&self, calls: impl MultiCall) -> PrioritizedMultiCall {
        PrioritizedMultiCall::new(calls, self.before_priority_fee_per_gas())
    }

    pub fn make_after(&self, calls: impl MultiCall) -> PrioritizedMultiCall {
        PrioritizedMultiCall::new(calls, self.priority_fee_per_gas())
    }
}

impl<'a, M: Middleware> AdjacentPendingTxsWithLogs<'a, M> {
    pub fn make_before_candidate<C: MultiCall + Clone>(
        &self,
        calls: C,
    ) -> CandidateToSend<'_, M, C> {
        self.block
            .make_candidate(calls, self.before_priority_fee_per_gas())
    }

    pub fn make_after_candidate<C: MultiCall + Clone>(
        &self,
        calls: C,
    ) -> CandidateToSend<'_, M, C> {
        self.block
            .make_candidate(calls, self.priority_fee_per_gas())
    }
}

#[derive(Default)]
pub struct ToSend(Mutex<Vec<PrioritizedMultiCall>>);

impl ToSend {
    pub async fn add_to_send(&self, calls: impl IntoIterator<Item = PrioritizedMultiCall>) {
        self.0.lock().await.extend(calls)
    }

    pub fn join_adjacent(self) -> Vec<PrioritizedMultiCall> {
        let mut calls: Vec<_> = self
            .0
            .into_inner()
            .into_iter()
            .group_by(|t| t.priority_fee_per_gas)
            .into_iter()
            .map(|(priority_fee, group)| PrioritizedMultiCall {
                calls: group.map(|g| g.calls).concat(),
                priority_fee_per_gas: priority_fee,
            })
            .collect();
        calls.sort_unstable_by_key(|p| Reverse(p.priority_fee_per_gas));
        calls
    }
}

pub struct CandidateToSend<'a, M, C: MultiCall> {
    call: PrioritizedMultiCall,
    function_call: MultiFunctionCall<Arc<M>, M, C>,
    block: &'a PendingBlock<'a, M>,
}

impl<'a, M, C: MultiCall> From<CandidateToSend<'a, M, C>> for PrioritizedMultiCall {
    fn from(candidate: CandidateToSend<'a, M, C>) -> Self {
        candidate.call
    }
}

impl<'a, M, C> CandidateToSend<'a, M, C>
where
    M: Middleware,
    C: MultiCall,
{
    pub async fn call(&self) -> Result<C::Ok, MultiCallContractError<M, C::Reverted>> {
        self.function_call.call().await
    }

    pub async fn estimate_fee(&self) -> Result<U256, MultiCallContractError<M, C::Reverted>> {
        let gas = self.function_call.estimate_gas().await?;
        Ok(gas * (self.block.base_fee_per_gas() + self.call.priority_fee_per_gas))
    }

    pub async fn add_to_send(self) {
        self.block.add_to_send(Some(self.call)).await
    }
}

#[autoimpl(Deref using self.block)]
pub struct PendingBlock<'a, M> {
    pub(crate) block: Block<TxWithLogs>,
    pub(crate) to_send: ToSend,
    account: Address, // TODO: get from client from multicall
    multicall: &'a MultiCallContract<Arc<M>, M>,
    first_priority_fee_per_gas: U256,
}

impl<'a, M> PendingBlock<'a, M> {
    pub fn try_from(
        block: Block<ethers::types::Transaction>,
        logs: impl IntoIterator<Item = Log>,
        first_prioroty_fee_per_gas: impl Into<Option<U256>>,
        account: Address,
        multicall: &'a MultiCallContract<Arc<M>, M>,
    ) -> Result<Self, InvalidTransaction> {
        // TODO: ensure all logs are from the same block as `block`
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
        Ok(Self {
            first_priority_fee_per_gas: 0.into(), // TODO
            // first_priority_fee_per_gas: first_prioroty_fee_per_gas.into().or_else(|| transactions.first().map(|tx| tx)), // TODO: or first tx
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
                        move |tx| {
                            Ok(TxWithLogs {
                                logs: logs
                                    .peeking_take_while(|l| {
                                        l.transaction_hash.is_some_and(|tx_hash| tx_hash == tx.hash)
                                    })
                                    .collect(),
                                tx: tx.try_into()?,
                            })
                        }
                    })
                    .try_collect()?,
                size,
                mix_hash,
                nonce,
                base_fee_per_gas,
                other,
            },
            to_send: Default::default(),
            account,
            multicall,
        })
    }

    pub fn account(&self) -> Address {
        self.account
    }

    fn base_fee_per_gas(&self) -> U256 {
        self.block.base_fee_per_gas.unwrap_or(0.into())
    }

    pub async fn add_to_send(&self, calls: impl IntoIterator<Item = PrioritizedMultiCall>) {
        self.to_send.add_to_send(calls).await
    }

    pub fn make_first_in_block(&self, calls: impl MultiCall) -> PrioritizedMultiCall {
        PrioritizedMultiCall::new(calls, self.first_priority_fee_per_gas)
    }

    pub fn iter_adjacent_txs(&self) -> impl Iterator<Item = AdjacentPendingTxsWithLogs<'_, M>> {
        self.block
            .transactions
            .group_by(|l, r| l.fees.priority_fee() == r.fees.priority_fee())
            .map(move |txs| AdjacentPendingTxsWithLogs { txs, block: &self })
    }
}

impl<'a, M> PendingBlock<'a, M>
where
    M: Middleware,
{
    fn make_candidate<C: MultiCall + Clone>(
        &self,
        calls: C,
        priority_fee_per_gas: U256,
    ) -> CandidateToSend<'_, M, C> {
        CandidateToSend {
            function_call: self
                .multicall
                .multicall(calls.clone())
                .block(BlockNumber::Pending), // TODO: priority_fee
            call: PrioritizedMultiCall::new(calls, priority_fee_per_gas),
            block: &self,
        }
    }

    pub fn make_first_in_block_candidate<C: MultiCall + Clone>(
        &self,
        calls: C,
    ) -> CandidateToSend<'_, M, C> {
        self.make_candidate(calls, self.first_priority_fee_per_gas)
    }
}

#[autoimpl(for<T: trait + ?Sized> &T,Arc<T>, Box<T>)]
pub trait PendingBlockMonitor<M: Middleware> {
    fn process_pending_block<'a>(
        &'a self,
        block: &'a PendingBlock<'a, M>,
    ) -> BoxFuture<'a, anyhow::Result<()>>;
}

impl<M: Middleware> PendingBlockMonitor<M> for () {
    fn process_pending_block<'a>(
        &'a self,
        _block: &'a PendingBlock<M>,
    ) -> BoxFuture<'a, anyhow::Result<()>> {
        future::ok(()).boxed()
    }
}

// pub struct LogMonitor;

// impl PendingBlockMonitor for LogMonitor {
//     fn process_pending_block<'a>(
//         &'a self,
//         block: &'a PendingBlock,
//     ) -> BoxFuture<'a, anyhow::Result<()>> {
//         debug!("")
//     }
// }

#[autoimpl(Deref using self.0)]
#[autoimpl(DerefMut using self.0)]
pub struct MultiMonitor<M>(Vec<M>);

impl<M> FromIterator<M> for MultiMonitor<M> {
    fn from_iter<T: IntoIterator<Item = M>>(monitors: T) -> Self {
        Self(monitors.into_iter().collect())
    }
}

impl<M> MultiMonitor<M> {
    pub fn into_inner(self) -> Vec<M> {
        self.0
    }
}

impl<MW, M> PendingBlockMonitor<MW> for MultiMonitor<M>
where
    MW: Middleware,
    M: PendingBlockMonitor<MW>,
{
    fn process_pending_block<'a>(
        &'a self,
        block: &'a PendingBlock<'a, MW>,
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

impl<MW, M> PendingBlockMonitor<MW> for TxMonitor<M>
where
    MW: Middleware,
    M: PendingTxMonitor,
{
    fn process_pending_block<'a>(
        &'a self,
        block: &'a PendingBlock<'a, MW>,
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

// pub trait ThinSandwichMonitor {
//     fn wrap_adjacent_pending_txs<'a>(
//         &'a self,
//         txs: &'a [TxWithLogs],
//     ) -> BoxFuture<'a, anyhow::Result<Option<[MultiCallGroups; 2]>>>;
// }

// pub struct ThinSandwichWrapper<M>(M);

// impl<MW, M> PendingBlockMonitor<MW> for ThinSandwichWrapper<M>
// where
//     M: ThinSandwichMonitor + Send + Sync,
//     MW: Middleware,
// {
//     fn process_pending_block<'a>(
//         &'a self,
//         block: &'a PendingBlock<'a, MW>,
//     ) -> BoxFuture<'a, anyhow::Result<()>> {
//         block
//             .iter_adjacent_txs()
//             .map(|txs| async move {
//                 let Some([before, after]) = self.0.wrap_adjacent_pending_txs(&txs).await? else {
//                     return Ok(())
//                 };
//                 txs.add_to_send_wrap(before, after).await;
//                 Ok(())
//             })
//             .collect::<FuturesUnordered<_>>()
//             .try_collect()
//             .boxed()
//     }
// }

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
