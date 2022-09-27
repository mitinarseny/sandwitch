#![feature(
    future_join,
    is_some_with,
    iterator_try_collect,
    result_flattening,
    result_option_inspect
)]
pub mod abort;
// mod buffer;
mod cached;
mod contracts;
mod metriced;
mod monitors;
mod timed;
mod with;

use self::abort::{AbortSet, FutureExt as AbortFutureExt};
use self::metriced::{FuturesUnordered, StreamExt as MetricedStreamExt};
use self::monitors::pancake_swap::{PancakeSwap, PancakeSwapConfig};
use self::monitors::{BlockMonitor, MultiTxMonitor, PendingTxMonitor, TxMonitor};
use self::timed::{
    StreamExt as TimedStreamExt, Timed, TryFutureExt as TimedTryFutureExt, TryTimedFuture,
};
use self::with::{FutureExt as WithFutureExt, With};

use std::sync::Arc;

use anyhow::{anyhow, Context};
use ethers::prelude::*;
use futures::future::join;
use futures::stream::FusedStream;
use futures::{
    future,
    future::{try_join, Aborted},
    stream::StreamExt,
    FutureExt, TryFutureExt,
};
use futures::{pin_mut, select_biased, Future, Stream};
use metrics::{register_counter, register_gauge, register_histogram, Counter, Gauge, Histogram};
use serde::Deserialize;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing::{error, error_span, info, trace, trace_span, warn, Instrument};
use url::Url;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub core: CoreConfig,
    pub monitors: MonitorsConfig,
}

#[derive(Deserialize, Debug)]
pub struct CoreConfig {
    pub wss: Url,
    pub http: Url,
}

#[derive(Deserialize, Debug)]
pub struct MonitorsConfig {
    pub pancake_swap: PancakeSwapConfig,
}

struct ProcessMetrics {
    seen_txs: Counter,
    resolved_txs: Counter,
    resolved_as_pending_txs: Counter,
    process_tx_duration: Histogram,
    missed_txs: Counter,

    height: Counter,
    process_block_duration: Histogram,
}

impl ProcessMetrics {
    fn new() -> Self {
        Self {
            seen_txs: register_counter!("sandwitch_seen_txs"),
            resolved_txs: register_counter!("sandwitch_resolved_txs"),
            resolved_as_pending_txs: register_counter!("sandwitch_resolved_as_pending_txs"),
            process_tx_duration: register_histogram!("sandwitch_process_tx_duration_seconds"),
            missed_txs: register_counter!("sandwitch_missed_txs"),
            height: register_counter!("sandwitch_height"),
            process_block_duration: register_histogram!("sandwitch_process_block_duration_seconds"),
        }
    }
}

pub struct App<SC, RC>
where
    SC: PubsubClient,
    RC: JsonRpcClient,
{
    streaming: Arc<Provider<SC>>,
    requesting: Arc<Provider<RC>>,
    monitors: RwLock<MultiTxMonitor<Box<dyn PendingTxMonitor>>>,
    process_metrics: ProcessMetrics,
}

impl App<Ws, Http> {
    pub async fn from_config(config: Config) -> anyhow::Result<Self> {
        Self::from_transports(
            Ws::connect(&config.core.wss)
                .await
                .inspect(|_| info!("web socket created"))?,
            Http::new(config.core.http.clone()),
            config,
        )
        .await
    }
}

impl<ST, RT> App<ST, RT>
where
    ST: PubsubClient,
    RT: JsonRpcClient + Clone + 'static,
{
    async fn from_transports(
        streaming: ST,
        requesting: RT,
        config: Config,
    ) -> anyhow::Result<Self> {
        let streaming = Arc::new(Provider::new(streaming));
        let requesting = Arc::new(Provider::new(requesting));

        let network_id = {
            let (streaming_net, requesting_net) =
                try_join(streaming.get_net_version(), requesting.get_net_version()).await?;
            if streaming_net != requesting_net {
                return Err(anyhow!("mismatching network IDs: streaming: {streaming_net}, requesting: {requesting_net}"));
            }
            streaming_net
        };
        info!(network_id);
        register_counter!("sandwitch_info", "network_id" => network_id).absolute(1);

        let pancake =
            PancakeSwap::from_config(requesting.clone(), config.monitors.pancake_swap).await?;
        Ok(Self {
            streaming,
            requesting,
            monitors: RwLock::new(MultiTxMonitor::new([
                Box::new(pancake) as Box<dyn PendingTxMonitor>
            ])),
            process_metrics: ProcessMetrics::new(),
        })
    }
}

impl<ST, RT> App<ST, RT>
where
    ST: PubsubClient,
    RT: JsonRpcClient,
{
    // pub async fn run1(&mut self, cancel_token: CancellationToken) -> anyhow::Result<()> {
    //     let new_blocks = self
    //         .streaming
    //         .subscribe_blocks()
    //         .inspect_ok(|_| info!("subscribed to new blocks"))
    //         .with_abort_unpin(cancel_token.cancelled().map(|_| Aborted))
    //         .await
    //         .with_context(|| "failed to subscribe to new blocks")??;
    //     let pending_tx_hashes = self
    //         .streaming
    //         .subscribe_pending_txs()
    //         .inspect_ok(|_| info!("subscribed to new pending transactions"))
    //         .with_abort_unpin(cancel_token.cancelled().map(|_| Aborted))
    //         .await
    //         .with_context(|| "failed to subscribe to new pending transactions")??;
    //
    //     let current_pending_txs = CancelSet::new();
    //     let resolve_buffer = register_gauge!("sandwitch_resolve_buffer");
    //     let process_buffer = register_gauge!("sandwitch_process_buffer");
    //     let send_buffer = register_gauge!("sandwitch_send_buffer");
    //
    //     let mut pending_txs = pending_tx_hashes
    //         .timed()
    //         .inspect({
    //             let seen_txs = register_counter!("sandwitch_seen_txs");
    //             move |tx_hash| {
    //                 seen_txs.increment(1);
    //                 trace!(tx_hash = ?*tx_hash, "new pending transaction");
    //             }
    //         })
    //         .filter_map(|tx_hash| async move {
    //             let cancel = match current_pending_txs.try_insert(*tx_hash).await {
    //                 Some(v) => v,
    //                 None => {
    //                     trace!(
    //                         tx_hash = ?*tx_hash,
    //                         "this transaction has already been seen, skipping..."
    //                     );
    //                     return None;
    //                 }
    //             };
    //             Some(
    //                 self.requesting
    //                     .get_transaction(*tx_hash)
    //                     .map_ok(move |o| o.map(move |tx| tx_hash.set(tx)))
    //                     .map(move |r| {
    //                         r.with_context(|| {
    //                             format!("failed to get transaction by hash: {:#x}", *tx_hash)
    //                         })
    //                     })
    //                     .try_timed()
    //                     .with_abort_unpin(cancel.cancelled())
    //                     .map(Result::ok),
    //             )
    //         })
    //         .inspect(|_| resolve_buffer.increment(1.0))
    //         .buffer_unordered(self.buffer_size)
    //         .inspect(|_| resolve_buffer.decrement(1.0))
    //         .filter_map(future::ready)
    //         .map_ok({
    //             let resolve_duration = register_histogram!("sandwitch_resolve_duration_seconds");
    //             move |tx| {
    //                 resolve_duration.record(tx.elapsed());
    //                 tx.into_inner()
    //             }
    //         })
    //         .try_filter_map(future::ok)
    //         .inspect_ok({
    //             let resolved_txs = register_counter!("sandwitch_resolved_txs");
    //             move |tx| {
    //                 resolved_txs.increment(1);
    //                 trace!(?tx.hash, "transaction resolved");
    //             }
    //         })
    //         .try_filter(|tx| {
    //             future::ready(
    //                 tx.block_hash.is_none()
    //                     && tx.block_number.is_none()
    //                     && tx.transaction_index.is_none(),
    //             )
    //         })
    //         .inspect_ok({
    //             let resolved_pending_txs = register_counter!("sandwitch_resolved_pending_txs");
    //             move |tx| {
    //                 resolved_pending_txs.increment(1);
    //                 trace!(?tx.hash, "resolved transaction is still pending");
    //             }
    //         })
    //         .try_filter(|tx| {
    //             future::ready(
    //                 !tx.value.is_zero()
    //                     && !tx.gas.is_zero()
    //                     && tx.gas_price.is_some_and(|g| !g.is_zero()),
    //             )
    //         })
    //         .map_ok({
    //             let monitors = &self.monitors;
    //             move |tx| {
    //                 let tx_hash = tx.hash;
    //                 async move {
    //                     let to_send = monitors.on_tx(&tx).try_timed().await?;
    //                     Ok((tx.hash, to_send))
    //                 }
    //                 .instrument(error_span!("process_tx", ?tx_hash))
    //             }
    //         })
    //         .inspect_ok(|_| process_buffer.increment(1.0))
    //         .try_buffer_unordered(self.buffer_size)
    //         .inspect_ok(|_| process_buffer.decrement(1.0))
    //         .try_filter_map({
    //             let current_pending_txs = &current_pending_txs;
    //             let missed_txs = register_counter!("sandwitch_missed_txs");
    //             let process_duration = register_histogram!("sandwitch_process_duration_seconds");
    //             move |(tx_hash, to_send)| {
    //                 process_duration.record(to_send.elapsed());
    //                 let missed_txs = missed_txs.clone();
    //                 async move {
    //                     current_pending_txs
    //                         .lock()
    //                         .await
    //                         .remove(&tx_hash)
    //                         .then(|| Ok(to_send.into_inner()))
    //                         .or_else(move || {
    //                             missed_txs.increment(1);
    //                             trace!(
    //                                 ?tx_hash,
    //                                 "this transaction has already been included in block"
    //                             );
    //                             None
    //                         })
    //                         .transpose()
    //                 }
    //             }
    //         })
    //         .try_filter(|to_send| future::ready(!to_send.is_empty()))
    //         .map_ok(|_to_send| future::ok(()).try_timed()) // TODO: send
    //         .inspect_ok(|_| send_buffer.increment(1.0))
    //         .try_buffer_unordered(self.buffer_size)
    //         .inspect_ok(|_| send_buffer.decrement(1.0))
    //         .map_ok({
    //             let send_duration = register_histogram!("sandwitch_send_duration_seconds");
    //             move |t| {
    //                 send_duration.record(t.elapsed());
    //                 t.into_inner()
    //             }
    //         })
    //         .boxed() // TODO: pin_mut!
    //         .fuse();
    //
    //     let mut new_blocks = new_blocks
    //         .filter_map(|block| future::ready(block.hash))
    //         .filter_map(|block_hash| {
    //             self.requesting
    //                 .get_block(block_hash.clone())
    //                 .map(move |r| {
    //                     r.with_context(|| format!("failed to get block by hash: {block_hash}"))
    //                 })
    //                 .map(Result::transpose)
    //         })
    //         .inspect_ok({
    //             let height = register_counter!("sandwitch_height");
    //             let last_block_tx_count = register_gauge!("sandwitch_last_block_tx_count");
    //             move |block| error_span!("update_block_metrics", block_hash = ?block.hash.unwrap())
    //                 .in_scope(|| {
    //                     if let Some(number) = block.number {
    //                         height.absolute(number.as_u64());
    //                     }
    //                     last_block_tx_count.set(block.transactions.len() as f64);
    //                     trace!(
    //                         tx_count = block.transactions.len(),
    //                         "new block",
    //                     );
    //                 })
    //         })
    //         .and_then({
    //             let current_pending_txs = &current_pending_txs;
    //             move |block| async move {
    //                 for h in current_pending_txs.cancel_iter(block.transactions).await {
    //                     trace!(
    //                         tx_hash = ?h,
    //                         "transaction has been included in new block, so cancel processing of this transaction",
    //                     );
    //                 }
    //                 Ok(block)
    //             }
    //             .instrument(error_span!("remove_txs_seen_in_block", block_hash = ?block.hash.unwrap()))
    //         })
    //         .boxed()
    //         .fuse();
    //
    //     let cancelled = cancel_token.cancelled().boxed().fuse();
    //
    //     loop {
    //         select_biased! {
    //             _ = cancelled => return Ok(()),
    //             block = new_blocks.try_next() => match block? {
    //                 Some(block) => {
    //                     self.monitors.on_block(&block)
    //                         .instrument(error_span!("on_block", block_hash = ?block.hash.unwrap()))
    //                         .await
    //                         .with_context(|| format!(
    //                             "failed to process block {:?}",
    //                             block.hash.unwrap(),
    //                         ))?;
    //                 },
    //                 None => return Err(anyhow!("new blocks stream finished unexpectedly")),
    //             },
    //             tx = pending_txs.try_next() => if tx?.is_none() {
    //                 return Err(anyhow!("new pending transactions stream finished unexpectedly"))
    //             },
    //         }
    //     }
    // }

    pub async fn run(&mut self, cancel: &CancellationToken) -> anyhow::Result<()> {
        let cancelled = cancel.cancelled();
        pin_mut!(cancelled);

        let (mut new_blocks, mut pending_tx_hashes) = try_join(
            self.streaming
                .subscribe_blocks()
                .inspect_ok(|st| info!(subscription_id = %st.id, "subscribed to new blocks"))
                .map(|r| r.with_context(|| "failed to subscribe to new blocks")),
            self.streaming
                .subscribe_pending_txs()
                .inspect_ok(
                    |st| info!(subscription_id = %st.id, "subscribed to new pending transactions"),
                )
                .map(|r| r.with_context(|| "failed to subscribe to new pending transactions")),
        )
        .with_abort(cancelled.map(|_| Aborted))
        .await??;

        let cancelled = cancel.cancelled();
        pin_mut!(cancelled);

        let r = self
            .process(
                new_blocks.by_ref(),
                pending_tx_hashes
                    .by_ref()
                    .take_until(cancelled.inspect(|_| info!("stopping transactions stream..."))),
            )
            .await;

        join(
            pending_tx_hashes.unsubscribe().map(|r| match r {
                Err(err) => error!(%err, "failed to unsubscribe from new pending transactions"),
                Ok(_) => info!("unsubscribed from new pending transactions"),
            }),
            new_blocks.unsubscribe().map(|r| match r {
                Err(err) => error!(%err, "failed to unsubscribe from new blocks"),
                Ok(_) => info!("unsubscribed from new blocks"),
            }),
        )
        .await;
        r
    }

    async fn process(
        &self,
        blocks: impl Stream<Item = Block<TxHash>> + Unpin,
        mut pending_txs: impl FusedStream<Item = TxHash> + Unpin,
    ) -> anyhow::Result<()> {
        let mut new_blocks = blocks
            .filter(|b| future::ready(b.hash.is_some() && b.number.is_some()))
            .fuse();

        let mut aborts = AbortSet::new();
        let mut processing_txs = FuturesUnordered::new(register_gauge!("sandwitch_in_flight_txs"));

        while !(pending_txs.is_terminated() && processing_txs.is_terminated()) {
            select_biased! {
                r = processing_txs.select_next_some() => self.process_tx_result(r)?,
                block = new_blocks.select_next_some() => {
                    aborts.abort_all().for_each(drop);

                    let block_hash = block.hash.unwrap();
                    let block_number = block.number.unwrap().as_u64();
                    self.process_metrics.height.absolute(block_number);
                    trace!(?block_hash, ?block_number, "got new block");
                    // TODO: metrics: gas avg (prcentil)

                    let process_block = self.process_block(&block).fuse();
                    pin_mut!(process_block);

                    loop {
                        select_biased! {
                            r = processing_txs.select_next_some() => self.process_tx_result(r)?,
                            r = process_block => {
                                break r;
                            },
                            tx_hash = pending_txs.select_next_some() => {
                                match self.maybe_process_tx(tx_hash, &mut aborts) {
                                    Some(f) => processing_txs.push(f),
                                    None => continue,
                                }
                            },
                        }
                    }?
                },
                tx_hash = pending_txs.select_next_some() => {
                    match self.maybe_process_tx(tx_hash, &mut aborts) {
                        Some(f) => processing_txs.push(f),
                        None => continue,
                    }
                },
                complete => break,
            }
        }
        Ok(())
    }

    fn maybe_process_tx(
        &self,
        tx_hash: TxHash,
        aborts: &mut AbortSet<TxHash>,
    ) -> Option<impl Future<Output = ProcessTxResult> + '_> {
        let abort = aborts.try_insert(tx_hash).or_else(|| {
            warn!(
                ?tx_hash,
                "this transaction has already been seen, skipping...",
            );
            None
        })?;
        self.process_metrics.seen_txs.increment(1);
        trace!(?tx_hash, "got new transaction");
        Some(
            self.process_tx(tx_hash)
                .try_timed()
                .with_abort_reg(abort)
                .with(tx_hash),
        )
    }

    #[tracing::instrument(skip_all, fields(?tx_hash))]
    async fn process_tx(&self, tx_hash: TxHash) -> anyhow::Result<()> {
        let tx = match self
            .requesting
            .get_transaction(tx_hash)
            .await
            .with_context(|| format!("failed to get transaction by hash: {tx_hash:#x}",))?
        {
            Some(tx) => tx,
            None => return Ok(()),
        };
        self.process_metrics.resolved_txs.increment(1);

        if tx.block_hash.is_some() || tx.block_number.is_some() || tx.transaction_index.is_some() {
            return Ok(());
        }
        self.process_metrics.resolved_as_pending_txs.increment(1);

        if tx.value.is_zero() || tx.gas.is_zero() || tx.gas_price.map_or(true, |g| g.is_zero()) {
            return Ok(());
        }

        let _to_send = self.monitors.read().await.on_tx(&tx).await?;
        // TODO: send in batch

        Ok(())
    }

    #[tracing::instrument(skip_all, fields(tx_hash = ?r.with()))]
    fn process_tx_result(&self, r: ProcessTxResult) -> anyhow::Result<()> {
        match r.into_inner() {
            Ok(r) => self
                .process_metrics
                .process_tx_duration
                .record(r?.elapsed()),
            Err(Aborted) => {
                self.process_metrics.missed_txs.increment(1);
                trace!("transaction has been missed");
            }
        }
        Ok(())
    }

    #[tracing::instrument(skip_all, block_hash = ?block.hash.unwrap())]
    async fn process_block(&self, block: &Block<TxHash>) -> anyhow::Result<()> {
        let r = self
            .monitors
            .write()
            .await
            .on_block(block)
            .try_timed()
            .await?;
        self.process_metrics
            .process_block_duration
            .record(r.elapsed());
        Ok(r.into_inner())
    }

    // #[tracing::instrument(skip_all, fields(block_hash = ?block.hash.unwrap()))]
    // async fn process_block(
    //     &self,
    //     block: Block<TxHash>,
    //     aborts: &mut AbortSet<TxHash>,
    //     processing_txs: FuturesUnordered<impl >
    // ) -> anyhow::Result<()> {
    //     aborts.abort_all().for_each(drop);
    //
    //     let block_number = block.number.unwrap().as_u64();
    //     self.process_metrics.height.absolute(block_number);
    //     trace!(block_number, "got new block");
    //
    //
    //     // TODO: metrics: gas avg (prcentil)
    //
    //     Ok(())
    // }
}

type ProcessTxResult = With<Result<anyhow::Result<Timed<()>>, Aborted>, TxHash>;
