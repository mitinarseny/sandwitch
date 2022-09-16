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

use self::abort::FutureExt as AbortFutureExt;
use self::monitors::pancake_swap::{PancakeSwap, PancakeSwapConfig};
use self::monitors::{Monitor, MultiTxMonitor};

use std::future;
use std::sync::Arc;

use anyhow::{anyhow, Context};
use ethers::prelude::*;
use futures::future::try_join;
use futures::select;
use futures::{future::Aborted, stream::StreamExt, FutureExt, TryFutureExt};
use metrics::{register_counter, register_gauge};
use serde::Deserialize;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, trace};
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

        let buffered_txs = register_gauge!("sandwitch_buffered_txs");
        let mut pending_txs = pending_tx_hashes
            .take_until(cancel_token.cancelled())
            .inspect({
                let seen_txs = register_counter!("sandwitch_seen_txs");
                move |tx_hash| {
                    seen_txs.increment(1);
                    trace!(?tx_hash, "new pending transaction");
                }
            })
            .map(|tx_hash| {
                self.requesting
                    .get_transaction(tx_hash)
                    .inspect_err(|err| error!(%err, "failed to get transaction by hash"))
                    .map(Result::ok)
                    .map(Option::flatten)
            })
            .inspect(|_| buffered_txs.increment(1.0))
            .buffer_unordered(self.buffer_size)
            .inspect(|_| buffered_txs.decrement(1.0))
            .filter_map(future::ready)
            .inspect({
                let resolved_txs = register_counter!("sandwitch_resolved_txs");
                move |tx| {
                    resolved_txs.increment(1);
                    trace!(?tx.hash, "transaction resolved");
                }
            })
            .filter(|tx| {
                future::ready(
                    tx.block_hash.is_none()
                        && tx.block_number.is_none()
                        && tx.transaction_index.is_none(),
                )
            })
            .inspect({
                let resolved_pending_txs = register_counter!("sandwitch_resolved_pending_txs");
                move |tx| {
                    resolved_pending_txs.increment(1);
                    trace!(?tx.hash, "resolved transaction is still pending");
                }
            })
            .boxed()
            .fuse();

        let mut new_blocks = new_blocks
            .filter(|block| future::ready(block.number.is_some() && block.hash.is_some()))
            .inspect({
                let height = register_counter!("sandwitch_height");
                move |block| {
                    height.absolute(block.number.unwrap().as_u64());
                    trace!(block_hash = ?block.hash.unwrap(), "new block");
                }
            })
            .fuse();

        loop {
            select! {
                tx = pending_txs.next() => match tx {
                    Some(tx) => {
                        self.monitors.process(&tx).await;
                    },
                    None => break,
                },
                block = new_blocks.next() => match block {
                    Some(block) => {
                        self.monitors.process(&block).await;
                    },
                    None => return Err(anyhow!("new blocks stream finished unexpectedly")),
                },
            }
        }

        Ok(())
    }
}
