use std::{collections::HashSet, sync::Arc};

use anyhow::Context;
use ethers::{
    providers::{Middleware, Provider, PubsubClient},
    signers::Signer,
    types::{Block, BlockNumber, Transaction, TxHash, H256},
};
use futures::{
    future::{try_join3, Aborted, Future, FutureExt, TryFutureExt},
    lock::Mutex,
    pin_mut, select_biased,
    stream::{FusedStream, FuturesUnordered, StreamExt, TryStreamExt},
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
    timed::TryFutureExt as TryTimedFutureExt,
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
            accounts: accounts.into(),
            client: client.into(),
            monitor: monitor.into(),
            metrics: Metrics::default().into(),
        }
    }

    pub(crate) async fn run(&mut self, cancel: CancellationToken) -> anyhow::Result<()> {
        let cancelled = cancel.cancelled().fuse();
        pin_mut!(cancelled);

        let (blocks, txs, last_block) = try_join3(
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
                .get_block(BlockNumber::Latest)
                .map_ok(Option::unwrap)
                .err_into::<anyhow::Error>(),
        )
        .with_abort(cancelled.map(|_| Aborted))
        .await??;

        let last_block_hash = Arc::new(Mutex::new(last_block.hash.unwrap()));

        let txs = txs.take_until(cancel.cancelled());
        pin_mut!(txs);

        let mut blocks = blocks
            .map(Ok)
            .try_filter_map(|block| {
                let block_hash = block.hash.unwrap();
                self.client
                    .get_block_with_txs(block_hash)
                    .try_timed()
                    .map_ok({
                        let metrics = &self.metrics;
                        move |(block, elapsed)| {
                            metrics.block_resolved(elapsed);
                            if block.is_none() {
                                warn!(?block_hash, "invalid block, skipping...");
                            }
                            block
                        }
                    })
            })
            .fuse();

        let mut handles = JoinHandleSet::default();

        let cancelled = cancel.cancelled().fuse();
        pin_mut!(cancelled);

        let mut first_err = None;
        while !(txs.is_terminated() && handles.is_terminated()) {
            select_biased! {
                _ = cancelled => {
                    handles.abort_all();
                },
                block = blocks.select_next_some() => {
                    let block = block?;
                    *last_block_hash.lock().await = block.hash.unwrap();
                    tokio::spawn(self.process_block(block, &mut handles)); // TODO: wait?
                },
                r = handles.select_next_some() => if let Err(err) = r {
                    error!(%err, "transaction processing failed");
                    if first_err.is_none() {
                        cancel.cancel();
                        first_err = Some(err);
                    }
                },
                tx_hash = txs.select_next_some() => {
                    self.maybe_process_tx(tx_hash, &mut handles, &last_block_hash)
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
    fn process_block(
        &self,
        block: Block<Transaction>,
        handles: &mut JoinHandleSet<TxHash, anyhow::Result<Option<M::Ok>>>,
    ) -> impl Future<Output = ()> {
        trace!("got new block");
        self.metrics.block_valid(&block);

        block
            .transactions
            .into_iter()
            .rev() // reverse so that we set only last nonces for accounts
            .inspect(|tx| {
                if handles.abort(&tx.hash).is_some() {
                    trace!(
                        ?tx.hash,
                        "transaction has just been mined, cancelling its processing...",
                    );
                    self.metrics.tx_missed();
                };
            })
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
            .inspect(|_| {
                trace!("block processed");
            })
            .in_current_span()
    }

    #[tracing::instrument(skip_all, fields(?tx_hash))]
    fn maybe_process_tx(
        &self,
        tx_hash: TxHash,
        handles: &mut JoinHandleSet<TxHash, anyhow::Result<Option<M::Ok>>>,
        last_block_hash: &Arc<Mutex<H256>>,
    ) {
        trace!("got new transaction");
        self.metrics.new_tx();

        let handle_entry = match handles.try_insert(tx_hash) {
            Ok(v) => v,
            Err(tx_hash) => {
                warn!(
                    ?tx_hash,
                    "this transaction is already being processed, skipping...",
                );
                return;
            }
        };

        handle_entry.spawn(Self::process_tx(
            self.client.clone(),
            self.monitor.clone(),
            tx_hash,
            last_block_hash.clone(),
            self.metrics.clone(),
        ));
    }

    #[tracing::instrument(skip_all, fields(?tx_hash, block_hash = field::Empty))]
    async fn process_tx(
        client: Arc<Provider<P>>,
        monitor: Arc<M>,
        tx_hash: H256,
        block_hash: Arc<Mutex<H256>>,
        metrics: Arc<Metrics>,
    ) -> anyhow::Result<Option<M::Ok>> {
        // TODO: timeouts
        let (tx, elapsed) = client
            .get_transaction(tx_hash)
            .try_timed()
            .await
            .with_context(|| "failed to get transaction")?;
        metrics.tx_resolved(elapsed);

        let Some(tx) = tx else {
            trace!("invalid tx, skipping...");
            return Ok(None);
        };
        metrics.tx_valid();
        if tx.block_hash.is_some() {
            trace!("transaction resolved as already mined, skipping...");
            return Ok(None);
        }
        metrics.tx_resolved_as_pending();

        let block_hash = *block_hash.lock_owned().await;
        Span::current().record("block_hash", format!("{block_hash:x}"));
        trace!("transaction resolved as pending, processing...");

        let (r, elapsed) = monitor.process_tx(&tx, block_hash).try_timed().await?;
        trace!("transaction processed");
        metrics.tx_processed(elapsed);

        Ok(Some(r))
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

    fn block_resolved(&self, elapsed: Duration) {
        self.resolve_block_duration.record(elapsed);
    }

    fn block_valid(&self, block: &Block<Transaction>) {
        self.height.absolute(block.number.unwrap().as_u64());
        self.txs_in_block.record(block.transactions.len() as f64);
    }
}
