use core::{cmp::Reverse, iter::Map, mem, slice};
use std::sync::Arc;

use ethers::{
    providers::Middleware,
    types::{Address, Block, BlockNumber, Log, TxHash, U256},
};
use futures::lock::Mutex;
use impl_tools::autoimpl;
use itertools::Itertools;
use sandwitch_contracts::{
    multicall::{
        Call, Calls, DynTryCall, MultiCall, MultiCallContract, MultiCallErrors, MultiFunctionCall,
        RawCall,
    },
    ContractError,
};
use thiserror::Error as ThisError;

use crate::transactions::{InvalidTransaction, Transaction};

#[derive(Debug)]
#[autoimpl(Deref using self.block)]
pub struct ProcessingBlock<M, TX = Transaction> {
    pub(crate) block: Block<TX>,
    pub(crate) to_send: ToSend,
    account: Address,
    multicall: Arc<MultiCallContract<Arc<M>, M>>,
    first_priority_fee_per_gas: U256,
}

impl<M, TX> ProcessingBlock<M, TX> {
    pub fn account(&self) -> Address {
        self.account
    }

    fn base_fee_per_gas(&self) -> U256 {
        self.block.base_fee_per_gas.unwrap_or(0.into())
    }

    pub async fn add_to_send(&self, calls: impl IntoIterator<Item = PrioritizedMultiCall>) {
        self.to_send.add_to_send(calls).await
    }

    pub fn first_in_block(&self, calls: impl MultiCall) -> PrioritizedMultiCall {
        PrioritizedMultiCall::new(calls, self.first_priority_fee_per_gas)
    }
}

impl<M, TX> ProcessingBlock<M, TX>
where
    M: Middleware,
{
    fn candidate<C: MultiCall + Clone>(
        &self,
        calls: C,
        priority_fee_per_gas: U256,
    ) -> CandidateToSend<'_, M, C, TX> {
        CandidateToSend {
            function_call: self
                .multicall
                .multicall(calls.clone())
                .block(BlockNumber::Pending)
                .priority_fee_per_gas(priority_fee_per_gas),
            call: PrioritizedMultiCall::new(calls, priority_fee_per_gas),
            block: &self,
        }
    }

    pub fn first_in_block_candidate<C: MultiCall + Clone>(
        &self,
        calls: C,
    ) -> CandidateToSend<'_, M, C, TX> {
        self.candidate(calls, self.first_priority_fee_per_gas)
    }
}

pub type PendingBlock<M> = ProcessingBlock<M, TxWithLogs>;

impl<M> PendingBlock<M> {
    pub fn iter_adjacent_txs(&self) -> impl Iterator<Item = AdjacentPendingTxsWithLogs<'_, M>> {
        self.block
            .transactions
            .group_by(|l, r| l.fees.priority_fee() == r.fees.priority_fee())
            .map(move |txs| AdjacentPendingTxsWithLogs { txs, block: &self })
    }
}

#[derive(Default)]
pub struct PriorityFeeEstimator;

impl PriorityFeeEstimator {
    pub fn estimate(&mut self, observed: impl Into<Option<U256>>) -> U256 {
        // TODO
        observed.into().unwrap_or(0.into()) + 1
    }
}

#[derive(ThisError, Debug)]
pub enum InvalidPendingBlock {
    #[error("invalid transaction {:?}: {}", .tx_hash, .error)]
    InvalidTransaction {
        tx_hash: TxHash,
        error: InvalidTransaction,
    },
}

pub struct PendingBlockFactory<M> {
    account: Address,
    multicall: Arc<MultiCallContract<Arc<M>, M>>,
    fees: Mutex<PriorityFeeEstimator>,
}

impl<M> PendingBlockFactory<M> {
    pub fn new(account: Address, multicall: impl Into<Arc<MultiCallContract<Arc<M>, M>>>) -> Self {
        Self {
            account,
            multicall: multicall.into(),
            fees: Default::default(),
        }
    }

    // pub async fn make_block(&self, block: Block<TxHash>) -> ProcessingBlock<M, TxHash> {}

    pub async fn make_pending_block(
        &self,
        block: Block<ethers::types::Transaction>,
        logs: impl IntoIterator<Item = Log>,
    ) -> Result<PendingBlock<M>, InvalidPendingBlock> {
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
            withdrawals,
            withdrawals_root,
            other,
        } = block;

        let transactions: Vec<TxWithLogs> = transactions
            .into_iter()
            .map({
                let mut logs = logs.into_iter().peekable();
                move |tx| {
                    let tx_hash = tx.hash;
                    Ok(TxWithLogs {
                        logs: logs
                            .peeking_take_while(|l| {
                                l.transaction_hash.is_some_and(|tx_hash| tx_hash == tx.hash)
                            })
                            .collect(),
                        tx: tx.try_into().map_err(|error| {
                            InvalidPendingBlock::InvalidTransaction { tx_hash, error }
                        })?,
                    })
                }
            })
            .try_collect()?;

        Ok(ProcessingBlock {
            first_priority_fee_per_gas: self
                .fees
                .lock()
                .await
                .estimate(transactions.first().map(|tx| tx.fees.priority_fee())),
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
                transactions,
                size,
                mix_hash,
                nonce,
                base_fee_per_gas,
                withdrawals,
                withdrawals_root,
                other,
            },
            to_send: Default::default(),
            account: self.account,
            multicall: self.multicall.clone(),
        })
    }
}

#[derive(Default, Debug)]
#[autoimpl(Deref<Target = [DynTryCall<RawCall>]> using self.calls)]
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
            DynTryCall {
                allow_failure: true,
                call,
            }
        }))
    }
}

impl IntoIterator for MultiCallGroups {
    type Item = RawCall;
    type IntoIter =
        Map<<Calls<RawCall> as IntoIterator>::IntoIter, fn(DynTryCall<RawCall>) -> RawCall>;

    fn into_iter(self) -> Self::IntoIter {
        self.calls.into_iter().map(DynTryCall::into_call)
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
    block: &'a PendingBlock<M>,
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

    pub fn front_run(&self, calls: impl MultiCall) -> PrioritizedMultiCall {
        PrioritizedMultiCall::new(calls, self.before_priority_fee_per_gas())
    }

    pub fn back_run(&self, calls: impl MultiCall) -> PrioritizedMultiCall {
        PrioritizedMultiCall::new(calls, self.priority_fee_per_gas())
    }
}

impl<'a, M: Middleware> AdjacentPendingTxsWithLogs<'a, M> {
    pub fn front_run_candidate<C: MultiCall + Clone>(
        &self,
        calls: C,
    ) -> CandidateToSend<'_, M, C, TxWithLogs> {
        self.block
            .candidate(calls, self.before_priority_fee_per_gas())
    }

    pub fn back_run_candidate<C: MultiCall + Clone>(
        &self,
        calls: C,
    ) -> CandidateToSend<'_, M, C, TxWithLogs> {
        self.block.candidate(calls, self.priority_fee_per_gas())
    }
}

impl<'a, M> IntoIterator for &'a AdjacentPendingTxsWithLogs<'a, M> {
    type Item = &'a TxWithLogs;

    type IntoIter = slice::Iter<'a, TxWithLogs>;

    fn into_iter(self) -> Self::IntoIter {
        self.txs.iter()
    }
}

#[derive(Default, Debug)]
pub struct ToSend(Mutex<Vec<PrioritizedMultiCall>>);

impl ToSend {
    pub async fn add_to_send(&self, calls: impl IntoIterator<Item = PrioritizedMultiCall>) {
        self.0.lock().await.extend(calls)
    }

    pub fn reduce(self) -> Vec<PrioritizedMultiCall> {
        let mut calls: Vec<_> = self
            .0
            .into_inner()
            .into_iter()
            .group_by(|t| t.priority_fee_per_gas)
            .into_iter()
            .filter_map(|(priority_fee_per_gas, group)| {
                Some(PrioritizedMultiCall {
                    calls: group.map(|g| g.calls).reduce(|mut r, l| {
                        r.extend(l.calls.into_iter().map(DynTryCall::into_call));
                        r
                    })?,
                    priority_fee_per_gas,
                })
            })
            .collect();
        calls.sort_unstable_by_key(|p| Reverse(p.priority_fee_per_gas));
        calls
    }
}

pub struct CandidateToSend<'a, M, C: MultiCall, TX> {
    call: PrioritizedMultiCall,
    function_call: MultiFunctionCall<Arc<M>, M, C>,
    block: &'a ProcessingBlock<M, TX>,
}

impl<'a, M, C: MultiCall, TX> From<CandidateToSend<'a, M, C, TX>> for PrioritizedMultiCall {
    fn from(candidate: CandidateToSend<'a, M, C, TX>) -> Self {
        candidate.call
    }
}

impl<'a, M, C, TX> CandidateToSend<'a, M, C, TX>
where
    M: Middleware,
    C: MultiCall,
{
    pub async fn call(
        &self,
    ) -> Result<Result<C::Ok, MultiCallErrors<C::Reverted>>, ContractError<M>> {
        self.function_call.call().await
    }

    // TODO: this is based on this actual block, not latest one
    pub async fn estimate_fee(
        &self,
    ) -> Result<Result<U256, MultiCallErrors<C::Reverted>>, ContractError<M>> {
        Ok(self
            .function_call
            .estimate_gas()
            .await?
            .map(|gas| gas * (self.block.base_fee_per_gas() + self.call.priority_fee_per_gas)))
    }

    pub async fn add_to_send(self) {
        self.block.add_to_send(Some(self.call)).await
    }
}
