#![feature(
    future_join,
    is_some_with,
    iterator_try_collect,
    result_flattening,
    result_option_inspect
)]
pub mod abort;
mod cached;
mod contracts;
mod metriced;
mod monitors;
mod timed;
mod with;

use self::abort::{AbortSet, FutureExt as AbortFutureExt};
use self::metriced::{
    FuturesOrdered as MetricedFuturesOrdered, FuturesUnordered as MetricedFuturesUnordered,
};
use self::monitors::{
    pancake_swap::{PancakeSwap, PancakeSwapConfig},
    Monitor, StatelessBlockMonitor, TxMonitor,
};
use self::timed::TryFutureExt as TimedTryFutureExt;

use std::sync::Arc;

use anyhow::{anyhow, Context};
use ethers::prelude::*;
use futures::{
    future,
    future::{join, try_join, AbortRegistration, Aborted, FutureExt, TryFutureExt},
    pin_mut, select_biased,
    stream::{FusedStream, FuturesUnordered, Stream, StreamExt, TryStreamExt},
};
use metrics::{register_counter, register_gauge, register_histogram, Counter, Histogram};
use serde::Deserialize;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, trace, warn};
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
    monitors: Vec<Box<dyn Monitor>>,
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
            monitors: [Box::new(RwLock::new(pancake)) as Box<dyn Monitor>].into(),
            process_metrics: ProcessMetrics::new(),
        })
    }
}

impl<ST, RT> App<ST, RT>
where
    ST: PubsubClient,
    RT: JsonRpcClient,
{
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
        // TODO: filter out txs sent by us
        let mut new_blocks = blocks
            .filter(|b| future::ready(b.hash.is_some() && b.number.is_some()))
            .fuse();

        let mut aborts = AbortSet::new();
        let mut processing_txs =
            MetricedFuturesUnordered::new(register_gauge!("sandwitch_in_flight_txs"));
        let mut processing_blocks =
            MetricedFuturesOrdered::new(register_gauge!("sandwitch_in_flight_blocks"));

        while !(pending_txs.is_terminated()
            && processing_txs.is_empty()
            && processing_blocks.is_empty())
        {
            select_biased! {
                r = processing_txs.select_next_some() => r?,
                r = processing_blocks.select_next_some() => r?,
                block = new_blocks.select_next_some() => {
                    aborts.abort_all().for_each(drop);

                    let block_hash = block.hash.unwrap();
                    let block_number = block.number.unwrap().as_u64();
                    self.process_metrics.height.absolute(block_number);
                    trace!(?block_hash, ?block_number, "got new block");
                    // TODO: metrics: gas avg (prcentil)

                    processing_blocks.push_back(self.process_block(block));
                },
                tx_hash = pending_txs.select_next_some() => {
                    let abort = match aborts.try_insert(tx_hash) {
                        Some(v) => v,
                        None => {
                            warn!(
                                ?tx_hash,
                                "this transaction has already been seen, skipping...",
                            );
                            continue
                        },
                    };
                    self.process_metrics.seen_txs.increment(1);
                    trace!(?tx_hash, "got new transaction");

                    processing_txs.push(self.process_tx(tx_hash, abort));
                },
                complete => break,
            }
        }
        Ok(())
    }

    #[tracing::instrument(level = "ERROR", skip_all, fields(
        block_hash = ?block.hash.unwrap(),
        block_number = block.number.unwrap().as_u64(),
    ))]
    async fn process_block(&self, block: Block<TxHash>) -> anyhow::Result<()> {
        let r = self.monitors.on_block(&block).try_timed().await?;
        let elapsed = r.elapsed();
        self.process_metrics.process_block_duration.record(elapsed);
        trace!(elapsed_ms = elapsed.as_millis(), "block processed");
        Ok(r.into_inner())
    }

    #[tracing::instrument(level = "ERROR", skip_all, fields(?tx_hash))]
    async fn process_tx(&self, tx_hash: TxHash, abort: AbortRegistration) -> anyhow::Result<()> {
        match self
            .get_and_process_tx(tx_hash)
            .try_timed()
            .with_abort_reg(abort)
            .await
        {
            Ok(r) => {
                let r = r?;
                let elapsed = r.elapsed();
                self.process_metrics.process_tx_duration.record(elapsed);
                trace!(elapsed_ms = elapsed.as_millis(), "transaction processed");
                Ok(r.into_inner())
            }
            Err(Aborted) => {
                self.process_metrics.missed_txs.increment(1);
                trace!("transaction has been missed");
                Ok(())
            }
        }
    }

    async fn get_and_process_tx(&self, tx_hash: TxHash) -> anyhow::Result<()> {
        let tx = match self
            .requesting
            .get_transaction(tx_hash)
            .await
            .with_context(|| format!("failed to get transaction by hash: {tx_hash:#x}"))?
        {
            Some(tx) => tx,
            None => {
                trace!("fake transaction, skipping...");
                return Ok(());
            }
        };
        self.process_metrics.resolved_txs.increment(1);

        if let Some(block_hash) = tx.block_hash {
            trace!(?block_hash, "transaction resolved as already mined");
            return Ok(());
        }
        self.process_metrics.resolved_as_pending_txs.increment(1);
        trace!("transaction resolved as pending");

        if tx.value.is_zero() || tx.gas.is_zero() || tx.gas_price.map_or(true, |g| g.is_zero()) {
            trace!("transaction has been filtered out because of zero value/gas/gas_price");
            return Ok(());
        }

        let to_send = self.monitors.on_tx(&tx).await?;

        // TODO: send in batch
        // TODO: filter out txs sent by us
        to_send
            .into_iter()
            .map(|tx| tx.nonce(1))
            .map(|tx| self.requesting.send_transaction(tx, None))
            .collect::<FuturesUnordered<_>>()
            .map_ok(|_| ())
            .try_collect::<()>()
            .await
            .map_err(Into::into)
    }
}
