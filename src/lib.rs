#![feature(
    iterator_try_collect,
    result_option_inspect,
    future_join,
    result_flattening
)]
mod monitors;
mod timed;

use self::monitors::pancake_swap::{PancakeSwap, PancakeSwapConfig};
use self::monitors::MultiMonitor;

use crate::timed::Timed;

use anyhow::anyhow;
use anyhow::Context;
use futures::stream::{StreamExt, TryStreamExt};
use futures::{try_join, TryFutureExt};
use metrics::{register_counter, Counter};
use monitors::Monitor;
use serde::Deserialize;
use std::future;
use std::time::SystemTime;
use tracing::{error, info, trace};
use web3::transports::{Http, WebSocket};
use web3::types::{Transaction, TransactionId};
use web3::{DuplexTransport, Transport, Web3};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub core: CoreConfig,
    pub monitors: MonitorsConfig,
}

#[derive(Deserialize, Debug)]
pub struct CoreConfig {
    pub wss: String,
    pub http: String,

    #[serde(default)]
    pub buffer_size: usize,
}

#[derive(Deserialize, Debug)]
pub struct MonitorsConfig {
    pub pancake_swap: PancakeSwapConfig,
}

pub struct App<ST, RT>
where
    ST: DuplexTransport,
    RT: Transport,
{
    streaming: Web3<ST>,
    requesting: Web3<RT>,
    buffer_size: usize,
    monitors: MultiMonitor<Timed<Transaction>, Transaction>,

    metrics: Metrics,
}

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

impl App<WebSocket, Http> {
    pub async fn from_config(config: Config) -> anyhow::Result<Self> {
        Self::from_transports(
            web3::transports::WebSocket::new(&config.core.wss)
                .await
                .inspect(|_| info!("web socket created"))?,
            web3::transports::Http::new(&config.core.http)?,
            config,
        )
        .await
    }
}

impl<ST, RT> App<ST, RT>
where
    ST: DuplexTransport,
    RT: Transport + Send + Sync + 'static,
{
    async fn from_transports(streaming: ST, requesting: RT, config: Config) -> anyhow::Result<Self>
    where
        <RT as Transport>::Out: Send,
    {
        let streaming = Web3::new(streaming);
        let requesting = Web3::new(requesting);

        let network_id = {
            let (streaming_net, requesting_net) =
                try_join!(streaming.net().version(), requesting.net().version())?;
            if streaming_net != requesting_net {
                return Err(anyhow!(
                    "mismatching network IDs: streaming: {streaming_net}, requesting: {requesting_net}"
                ));
            }
            streaming_net
        };
        info!(network_id);
        register_counter!("sandwitch_info", "network_id" => network_id).absolute(1);

        let pancake =
            PancakeSwap::from_config(requesting.eth(), config.monitors.pancake_swap).await?;
        Ok(Self {
            streaming,
            requesting,
            buffer_size: config.core.buffer_size,
            monitors: MultiMonitor::new(config.core.buffer_size, vec![Box::new(pancake)]),
            metrics: Metrics::new(),
        })
    }
}

impl<ST, RT> App<ST, RT>
where
    ST: DuplexTransport + Send + Sync,
    <ST as DuplexTransport>::NotificationStream: Send,
    RT: Transport + Send,
    <RT as Transport>::Out: Send,
{
    pub async fn run(&mut self) -> anyhow::Result<()> {
        let pending_tx_hashes = self
            .streaming
            .eth_subscribe()
            .subscribe_new_pending_transactions()
            .await
            .with_context(|| "failed to subscribe to new pending transactions")?;

        info!("subscribed to new pending transactions");

        let pending_txs = pending_tx_hashes
            .inspect_err(|err| error!(%err, "error while fetching new pending transactions"))
            .map_ok({
                let eth = self.requesting.eth();
                let metrics = &self.metrics;
                move |tx_hash| {
                    metrics.seen_txs.increment(1);
                    trace!(?tx_hash, "new pending transaction hash");

                    let at = SystemTime::now();
                    eth.transaction(TransactionId::Hash(tx_hash))
                        .inspect_err(|err| error!(%err, "failed to get transaction by hash"))
                        .map_ok(move |r| {
                            r.map(move |tx| {
                                metrics.resolved_txs.increment(1);
                                trace!(?tx.hash, "pending transaction resolved");

                                Timed::with(tx, at)
                            })
                        })
                }
            })
            .try_buffer_unordered(self.buffer_size)
            .filter_map(|r| future::ready(r.unwrap_or(None)))
            .boxed();

        let mut send_txs = self.monitors.process(pending_txs);

        while let Some(tx) = send_txs.next().await {
            info!(?tx, "send tx");
        }
        Ok(())
    }
}
