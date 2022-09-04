#![feature(
    iterator_try_collect,
    result_option_inspect,
    future_join,
    result_flattening
)]
mod monitors;
mod timed;

use std::future;
use std::time::SystemTime;
use tracing::{error, info, trace};

use anyhow::Context;
use futures::{try_join, TryFutureExt};
use serde::Deserialize;

use monitors::Monitor;

use futures::stream::{StreamExt, TryStreamExt};
use web3::transports::{Http, WebSocket};
use web3::types::{Transaction, TransactionId};
use web3::{DuplexTransport, Transport, Web3};

use anyhow::anyhow;

use crate::timed::Timed;

use self::monitors::pancake_swap::{PancakeSwap, PancakeSwapConfig};
use self::monitors::MultiMonitor;

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

        let (streaming_net, requesting_net) =
            try_join!(streaming.net().version(), requesting.net().version())?;

        if streaming_net != requesting_net {
            return Err(anyhow!(
                "mismatching network IDs: streaming: {streaming_net}, requesting: {requesting_net}"
            ));
        }
        info!(network_id = streaming_net);

        let pancake =
            PancakeSwap::from_config(requesting.eth(), config.monitors.pancake_swap).await?;
        Ok(Self {
            streaming,
            requesting,
            buffer_size: config.core.buffer_size,
            monitors: MultiMonitor::new(config.core.buffer_size, vec![Box::new(pancake)]),
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
            .inspect_ok(|tx_hash| trace!(?tx_hash, "new pending transaction hash"))
            .map_ok({
                let eth = self.requesting.eth();
                move |h| {
                    let at = SystemTime::now();
                    eth.transaction(TransactionId::Hash(h))
                        .inspect_err(|err| error!(%err, "failed to get transaction by hash"))
                        .map_ok(move |r| {
                            r.inspect(|tx| trace!(?tx.hash, "pending transaction resolved"))
                                .map(move |tx| Timed::with(tx, at))
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
