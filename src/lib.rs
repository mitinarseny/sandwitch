#![feature(
    iterator_try_collect,
    result_option_inspect,
    future_join,
    result_flattening
)]
pub mod abort;
mod contracts;
mod monitors;
mod timed;

use self::monitors::pancake_swap::{PancakeSwap, PancakeSwapConfig};
use self::monitors::{Monitor, MultiTxMonitor};
use ethers::prelude::*;
use futures::future::try_join;
use futures::select;
use std::sync::Arc;
use url::Url;

use self::abort::FutureExt as AbortFutureExt;

use anyhow::{anyhow, Context};
use futures::{future::Aborted, stream::StreamExt, FutureExt, TryFutureExt};
use metrics::{register_counter, Counter};
use serde::Deserialize;
use std::future;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, trace};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub core: CoreConfig,
    pub monitors: MonitorsConfig,
}

#[derive(Deserialize, Debug)]
pub struct CoreConfig {
    pub wss: Url,
    pub http: Url,

    #[serde(default)]
    pub buffer_size: usize,
}

#[derive(Deserialize, Debug)]
pub struct MonitorsConfig {
    pub pancake_swap: PancakeSwapConfig,
}

pub struct App<SC, RC>
where
    SC: PubsubClient,
    RC: JsonRpcClient,
{
    streaming: Arc<Provider<SC>>,
    requesting: Arc<Provider<RC>>,
    buffer_size: usize,
    monitors: MultiTxMonitor,

    metrics: Metrics,
}

#[derive(Clone)]
struct Metrics {
    seen_txs: Counter,
    resolved_txs: Counter,
}

impl Metrics {
    fn new() -> Self {
        Self {
            seen_txs: register_counter!("sandwitch_seen_txs"),
            resolved_txs: register_counter!("sandwitch_resolved_txs"),
        }
    }
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
            buffer_size: config.core.buffer_size,
            monitors: MultiTxMonitor::new(vec![Box::new(pancake)]),
            metrics: Metrics::new(),
        })
    }
}

impl<ST, RT> App<ST, RT>
where
    ST: PubsubClient,
    RT: JsonRpcClient,
{
    pub async fn run(&mut self, cancel_token: CancellationToken) -> anyhow::Result<()> {
        let new_blocks = self
            .streaming
            .subscribe_blocks()
            .inspect_ok(|_| info!("subscribed to new blocks"))
            .with_abort_unpin(cancel_token.cancelled().map(|_| Aborted))
            .await
            .with_context(|| "failed to subscribe to new blocks")??;
        let pending_tx_hashes = self
            .streaming
            .subscribe_pending_txs()
            .inspect_ok(|_| info!("subscribed to new pending transactions"))
            .with_abort_unpin(cancel_token.cancelled().map(|_| Aborted))
            .await
            .with_context(|| "failed to subscribe to new pending transactions")??;

        let mut pending_txs = pending_tx_hashes
            .take_until(cancel_token.cancelled())
            .map({
                let requesting = &self.requesting;
                let Metrics {
                    seen_txs,
                    resolved_txs,
                } = self.metrics.clone();
                move |tx_hash| {
                    seen_txs.increment(1);
                    trace!(?tx_hash, "new pending transaction");

                    // let at = SystemTime::now();
                    requesting
                        .get_transaction(tx_hash)
                        .inspect_err(|err| error!(%err, "failed to get transaction by hash"))
                        .map_ok({
                            let resolved_txs = resolved_txs.clone();
                            move |r| {
                                r.filter(|tx| {
                                    tx.block_hash.is_none()
                                        && tx.block_number.is_none()
                                        && tx.transaction_index.is_none()
                                })
                                .map(move |tx| {
                                    resolved_txs.increment(1);
                                    trace!(?tx.hash, "pending transaction resolved");

                                    // Timed::with(tx, at) TODO
                                    tx
                                })
                            }
                        })
                        .map(Result::ok)
                        .map(Option::flatten)
                }
            })
            .buffer_unordered(self.buffer_size)
            .filter_map(future::ready)
            .boxed()
            .fuse();
        let mut new_blocks = new_blocks.fuse();

        loop {
            select! {
                tx = pending_txs.next() => match tx {
                    Some(tx) => {
                        self.monitors.process(tx).await;
                    },
                    None => break,
                },
                block = new_blocks.next() => match block {
                    Some(block) => {
                        trace!(?block.hash, "new block");
                        self.monitors.process(block).await;
                    },
                    None => return Err(anyhow!("blocks finished")), // TODO
                },
            }
        }

        Ok(())
    }
}
