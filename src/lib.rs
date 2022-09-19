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

use std::collections::HashSet;
use std::future;
use std::sync::Arc;

use anyhow::{anyhow, Context};
use ethers::prelude::*;
use futures::{
    future::{try_join, Aborted},
    lock::Mutex,
    select,
    stream::StreamExt,
    FutureExt, TryFutureExt,
};
use metrics::{register_counter, register_gauge};
use serde::Deserialize;
use tokio_util::sync::CancellationToken;
use tracing::{error, error_span, info, trace, warn, Instrument};
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

        let current_pending_txs: Mutex<HashSet<H256>> = Mutex::new(HashSet::new());
        let resolve_buffer = register_gauge!("sandwitch_resolve_buffer");
        let process_buffer = register_gauge!("sandwitch_process_buffer");
        let send_buffer = register_gauge!("sandwitch_send_buffer");

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
                    .get_transaction(tx_hash.clone())
                    .inspect_err(|err| error!(%err, "failed to get transaction by hash"))
                    .map(Result::ok)
                    .map(Option::flatten)
                    .instrument(error_span!("get_transaction_by_hash", ?tx_hash))
            })
            .inspect(|_| resolve_buffer.increment(1.0))
            .buffer_unordered(self.buffer_size)
            .inspect(|_| resolve_buffer.decrement(1.0))
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
            .filter({
                let current_pending_txs = &current_pending_txs;
                move |tx| {
                    let tx_hash = tx.hash.clone();
                    async move {
                        let first_seen = current_pending_txs.lock().await.insert(tx_hash);
                        if !first_seen {
                            warn!(
                                ?tx_hash,
                                "this transaction has already been seen, skipping..."
                            );
                        }
                        first_seen
                    }
                }
            })
            .map({
                let monitors = &self.monitors;
                move |tx| {
                    let tx_hash = tx.hash.clone();
                    async move {
                        let to_send = monitors.on_tx(&tx).await;
                        (tx.hash, to_send)
                    }
                    .instrument(error_span!("process_tx", ?tx_hash))
                }
            })
            .inspect(|_| process_buffer.increment(1.0))
            .buffer_unordered(self.buffer_size)
            .inspect(|_| process_buffer.decrement(1.0))
            .filter_map({
                let current_pending_txs = &current_pending_txs;
                let missed_txs = register_counter!("sandwitch_missed_txs");
                move |(tx_hash, to_send)| {
                    let missed_txs = missed_txs.clone();
                    async move {
                        current_pending_txs
                            .lock()
                            .await
                            .remove(&tx_hash)
                            .then_some(to_send)
                            .or_else(move || {
                                missed_txs.increment(1);
                                trace!(
                                    ?tx_hash,
                                    "this transaction has already been included in block"
                                );
                                None
                            })
                    }
                }
            })
            .filter(|to_send| future::ready(!to_send.is_empty()))
            .map(|_to_send| future::ready(())) // TODO: send
            .inspect(|_| send_buffer.increment(1.0))
            .buffer_unordered(self.buffer_size)
            .inspect(|_| send_buffer.decrement(1.0))
            .boxed()
            .fuse();

        let mut new_blocks = new_blocks
            .filter_map(|block| future::ready(block.hash))
            .filter_map(|block_hash| {
                self.requesting
                    .get_block(block_hash.clone())
                    .inspect_err(|err| error!(%err, "failed to get block by hash"))
                    .map(Result::ok)
                    .map(Option::flatten)
                    .instrument(error_span!("get_block_by_hash", ?block_hash))
            })
            .inspect({
                let height = register_counter!("sandwitch_height");
                let last_block_tx_count = register_gauge!("sandwitch_last_block_tx_count");
                move |block| error_span!("update_block_metrics", block_hash = ?block.hash.unwrap())
                    .in_scope(|| {
                        if let Some(number) = block.number {
                            height.absolute(number.as_u64());
                        }
                        last_block_tx_count.set(block.transactions.len() as f64);
                        trace!(
                            tx_count = block.transactions.len(),
                            "new block",
                        );
                    })
            })
            .then({
                let current_pending_txs = &current_pending_txs;
                move |block| {
                    let block_hash = block.hash;
                    async move {
                        let mut txs = current_pending_txs.lock().await;
                        for h in block.transactions.iter() {
                            if txs.remove(h) {
                                trace!(tx_hash = ?h, "transaction has been included in new block, so removing from current pending set...");
                            };
                            // TODO: what about chain reorderings?
                        }
                        block
                    }
                    .instrument(error_span!("remove_txs_seen_in_block", ?block_hash))
                }
            })
            .boxed()
            .fuse();

        loop {
            select! {
                tx = pending_txs.next() => if tx.is_none() {
                    break;
                },
                block = new_blocks.next() => match block {
                    Some(block) => {
                        self.monitors.on_block(&block).await;
                    },
                    None => return Err(anyhow!("new blocks stream finished unexpectedly")),
                },
            }
        }

        Ok(())
    }
}
