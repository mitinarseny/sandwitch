use std::{collections::HashSet, sync::Arc};

use anyhow::Context;
use ethers::{
    abi::AbiEncode,
    providers::{Middleware, Provider, ProviderError, PubsubClient},
    signers::Signer,
    types::{Block, BlockNumber, Transaction, TxHash, H256},
};
use futures::{
    future::{try_join3, Aborted, FusedFuture, Future, FutureExt, TryFuture, TryFutureExt},
    lock::Mutex,
    pin_mut, select_biased,
    stream::{FusedStream, FuturesOrdered, FuturesUnordered, StreamExt},
};
use metrics::{register_counter, register_histogram, Counter, Histogram};
use tokio::{
    self,
    time::{timeout, Duration},
};
use tokio_util::sync::CancellationToken;
use tracing::{error, field, info, trace, warn, Instrument, Span};

use crate::{
    abort::{FutureExt as AbortFutureExt, JoinHandleSet},
    accounts::Accounts,
    monitors::TxMonitor,
    timed::{FutureExt as TimedFutureExt, TryFutureExt as TryTimedFutureExt},
};

pub(crate) struct Engine<P, S, M>
where
    P: PubsubClient,
    S: Signer,
    M: TopTxMonitor,
{
    client: Arc<Provider<P>>,
    accounts: Arc<Accounts<P, S>>,
    monitor: Arc<M>,
    metrics: Arc<Metrics>,
}

impl<P, S, M> Engine<P, S, M>
where
    P: PubsubClient + 'static,
    S: Signer + 'static,
    M: TopTxMonitor,
{
    pub(crate) fn new(
        client: impl Into<Arc<Provider<P>>>,
        accounts: impl Into<Arc<Accounts<P, S>>>,
        monitor: impl Into<Arc<M>>,
    ) -> Self {
        Self {
            client: client.into(),
            accounts: accounts.into(),
            monitor: monitor.into(),
            metrics: Metrics::default().into(),
        }
    }

    pub(crate) async fn run(&mut self, cancel: CancellationToken) -> anyhow::Result<()> {
        let cancelled = cancel.cancelled().fuse();
        pin_mut!(cancelled);

        let (blocks, tx_hashes, last_block) = try_join3(
            self.client
                .subscribe_blocks()
                .inspect_ok(|st| info!(subscription_id = %st.id, "subscribed to new blocks"))
                .map(|r| r.with_context(|| "failed to subscribe to new blocks")),
            self.client
                .subscribe_pending_txs()
                .inspect_ok(
                    |st| info!(subscription_id = %st.id, "subscribed to new pending transactions"),
                )
                .map(|r| r.with_context(|| "failed to subscribe to new pending transactions")),
            self.client
                .get_block_with_txs(BlockNumber::Latest)
                .map_ok(Option::unwrap)
                .err_into::<anyhow::Error>(),
        )
        .with_abort(cancelled.map(|_| Aborted))
        .await??;

        let mut last_block_hash = last_block.hash.unwrap();
        self.metrics.new_block(&last_block);

        let tx_hashes = tx_hashes.take_until(cancel.cancelled());
        let mut txs = FuturesUnordered::new();
        let mut process_tx_handles =
            JoinHandleSet::<TxHash, Result<(M::Ok, Duration), M::Error>>::default();

        let blocks = blocks.take_until(cancel.cancelled());
        let mut blocks_with_txs = FuturesOrdered::new();
        let mut process_block_handles = JoinHandleSet::default();

        let cancelled = cancel.cancelled().fuse();
        pin_mut!(tx_hashes, blocks, cancelled);

        let mut first_err: Option<anyhow::Error> = None;
        while !(cancelled.is_terminated()
            && process_tx_handles.is_terminated()
            && process_block_handles.is_terminated())
        {
            select_biased! {
                _ = cancelled => {
                    process_tx_handles.abort_all();
                    process_block_handles.abort_all();
                },
                r = process_block_handles.select_next_some() => match r {
                    Ok(((_, elapsed), block_hash)) => {
                        trace!(?block_hash, ?elapsed, "block proceesed");
                        self.metrics.block_processed(elapsed);
                    },
                    Err(err) => {
                        error!(%err, "block processing failed");
                        if first_err.is_none() {
                            cancel.cancel();
                            first_err = Some(err.into());
                        }
                        continue;
                    },
                },
                r = process_tx_handles.select_next_some() => {
                    match r.map(|(r, tx_hash)| r.map(move |v| (v, tx_hash)))
                        .map_err(Into::into).flatten() {
                        Ok(((_, elapsed), tx_hash)) => {
                            trace!(?tx_hash, ?elapsed, "transaction processed");
                            self.metrics.tx_processed(elapsed);
                        },
                        Err(err) => {
                            error!(%err, "transaction processing failed");
                            if first_err.is_none() {
                                cancel.cancel();
                                first_err = Some(err);
                            }
                            continue;
                        },
                    }
                },
                block = blocks_with_txs.select_next_some() => match block {
                    Ok(((block, block_hash), elapsed)) => {
                        trace!(?block_hash, ?elapsed, "block resolved");
                        self.metrics.block_resolved(elapsed);

                        let Some(block): Option<Block<Transaction>> = block else {
                            trace!(?block_hash, "invalid block, skipping...");
                            continue;
                        };
                        self.metrics.block_valid(&block);

                        let Ok(h) = process_block_handles.try_insert(block_hash) else {
                            warn!(
                                ?block_hash,
                                "this block is already being processed, skipping...",
                            );
                            continue;
                        };

                        for tx in &block.transactions {
                            if process_tx_handles.abort(&tx.hash).is_some() {
                                trace!(
                                    ?tx.hash,
                                    ?block_hash,
                                    "transaction has just been mined, cancelling its processing...",
                                );
                                self.metrics.tx_missed();
                            };
                        }

                        h.spawn(self.process_block(block).timed());
                    },
                    Err(err) => {
                        error!(%err, "failed to get block with txs");
                        if first_err.is_none() {
                            cancel.cancel();
                            first_err = Some(err);
                        }
                        continue;
                    },
                },
                block = blocks.select_next_some() => {
                    let block_number = block.number.unwrap();
                    last_block_hash = block.hash.unwrap();
                    trace!(
                        block_hash = ?last_block_hash,
                        block_number = block_number.as_u64(),
                        "got new block",
                    );
                    self.metrics.new_block(&block);

                    blocks_with_txs.push_back(
                        self.client
                            .get_block_with_txs(last_block_hash)
                            .map_ok(move |b| (b, last_block_hash))
                            .err_into()
                            .try_timed(),
                    );
                },
                tx = txs.select_next_some() => match tx {
                    Ok(((tx, tx_hash), elapsed)) => {
                        self.metrics.tx_resolved(elapsed);
                        trace!(?tx_hash, ?elapsed, "transaction resolved");

                        let Some(tx): Option<Transaction> = tx else {
                            trace!(?tx_hash, "invalid tx, skipping...");
                            continue;
                        };
                        self.metrics.tx_valid();

                        if let Some(block_hash) = tx.block_hash {
                            trace!(
                                ?tx_hash,
                                ?block_hash,
                                "transaction resolved as already mined, skipping...",
                            );
                            continue;
                        }
                        self.metrics.tx_resolved_as_pending();

                        let Ok(h) = process_tx_handles.try_insert(tx_hash) else {
                            warn!(
                                ?tx_hash,
                                "this transaction is already being processed, skipping...",
                            );
                            continue;
                        };

                        trace!(
                            ?tx_hash,
                            ?last_block_hash,
                            "transaction resolved as pending, processing...",
                        );
                        // TODO: another counter
                        h.spawn(self.process_tx(tx, last_block_hash).into_future().try_timed());
                    },
                    Err(err) => {
                        error!(%err, "failed to resolve tx");
                        if first_err.is_none() {
                            cancel.cancel();
                            first_err = Some(err);
                        }
                        continue;
                    },
                },
                tx_hash = tx_hashes.select_next_some() => {
                    trace!(?tx_hash, "got new transaction hash");
                    self.metrics.new_tx();
                    txs.push(
                        self.client
                            .get_transaction(tx_hash)
                            .map_ok(move |tx| (tx, tx_hash))
                            .err_into()
                            .try_timed(),
                    );
                },
            }
        }

        first_err.map_or(Ok(()), Err).map_err(Into::into)
    }

    #[tracing::instrument(
        skip_all,
        fields(
            block_hash = ?block.hash.unwrap(),
            block_number = block.number.unwrap().as_u64(),
        ),
    )]
    fn process_block(&self, block: Block<Transaction>) -> impl Future<Output = ()> {
        block
            .transactions
            .into_iter()
            .rev()
            .filter_map({
                let mut seen = HashSet::new();
                move |tx| {
                    if let Some(account) = self.accounts.get(&tx.from) {
                        if seen.insert(account.address()) {
                            return Some(account.lock().map(move |mut a| a.tx_mined(&tx)));
                        }
                    }
                    None
                }
            })
            .collect::<FuturesUnordered<_>>()
            .collect::<()>()
            .in_current_span()
    }

    #[tracing::instrument(skip_all, fields(?tx.hash, ?block_hash))]
    fn process_tx(
        &self,
        tx: Transaction,
        block_hash: H256,
    ) -> impl TryFuture<Ok = M::Ok, Error = M::Error> {
        let monitor = self.monitor.clone();
        async move { monitor.process_tx(&tx, block_hash).await }
    }
}

pub(crate) trait TopTxMonitor:
    TxMonitor<Ok = (), Error = anyhow::Error> + Sync + Send + 'static
{
}

impl<M> TopTxMonitor for M where M: TxMonitor<Ok = (), Error = anyhow::Error> + Sync + Send + 'static
{}

struct Metrics {
    seen_txs: Counter,
    resolve_tx_duration: Histogram,
    valid_txs: Counter,
    resolved_as_pendning_txs: Counter,
    process_tx_duration: Histogram,
    missed_txs: Counter,

    height: Counter,
    resolve_block_duration: Histogram,
    txs_in_block: Histogram,
    process_block_duration: Histogram,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            seen_txs: register_counter!("sandwitch_seen_txs"),
            resolve_tx_duration: register_histogram!("sandwitch_resolve_tx_duration"),
            valid_txs: register_counter!("sandwitch_valid_txs"),
            resolved_as_pendning_txs: register_counter!("sandwitch_resolved_as_pending_txs"),
            process_tx_duration: register_histogram!("sandwitch_process_tx_duration"),
            missed_txs: register_counter!("sandwitch_missed_txs"),
            height: register_counter!("sandwitch_height"),
            resolve_block_duration: register_histogram!("sandwitch_resolve_block_duration"),
            txs_in_block: register_histogram!("sandwitch_txs_in_block"),
            process_block_duration: register_histogram!("sandwitch_process_block_duration"),
        }
    }
}

impl Metrics {
    fn new_tx(&self) {
        self.seen_txs.increment(1);
    }

    fn tx_resolved(&self, elapsed: Duration) {
        self.resolve_tx_duration.record(elapsed)
    }

    fn tx_valid(&self) {
        self.valid_txs.increment(1);
    }

    fn tx_resolved_as_pending(&self) {
        self.resolved_as_pendning_txs.increment(1);
    }

    fn tx_processed(&self, elapsed: Duration) {
        self.process_tx_duration.record(elapsed);
    }

    fn tx_missed(&self) {
        self.missed_txs.increment(1);
    }

    fn new_block<TX>(&self, block: &Block<TX>) {
        self.height.absolute(block.number.unwrap().as_u64());
    }

    fn block_resolved(&self, elapsed: Duration) {
        self.resolve_block_duration.record(elapsed);
    }

    fn block_valid<TX>(&self, block: &Block<TX>) {
        self.txs_in_block.record(block.transactions.len() as f64);
    }

    fn block_processed(&self, elapsed: Duration) {
        self.process_block_duration.record(elapsed);
    }
}
