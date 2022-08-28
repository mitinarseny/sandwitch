#![feature(iterator_try_collect, result_option_inspect, future_join)]
mod monitors;
mod types;

use std::future;
use std::sync::Arc;

use anyhow::Context;
use serde::Deserialize;

use monitors::Monitor;

use futures::stream::{Stream, StreamExt, TryStream, TryStreamExt};
use web3::transports::{Http, WebSocket};
use web3::types::{Transaction, TransactionId, H256};
use web3::{DuplexTransport, Transport, Web3};

// use self::monitors::logger::Logger;
use self::monitors::pancake_swap::{PancakeSwap, PancakeSwapConfig};

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
    monitor: Arc<PancakeSwap<RT>>,
}

impl App<WebSocket, Http> {
    pub async fn from_config(config: Config) -> anyhow::Result<Self> {
        Self::from_transports(
            web3::transports::WebSocket::new(&config.core.wss).await?,
            web3::transports::Http::new(&config.core.http)?,
            config,
        )
        .await
    }
}

impl<ST, RT> App<ST, RT>
where
    ST: DuplexTransport,
    RT: Transport,
{
    async fn from_transports(
        streaming: ST,
        requesting: RT,
        config: Config,
    ) -> anyhow::Result<Self> {
        let requesting = Web3::new(requesting);
        let pancake =
            PancakeSwap::from_config(requesting.eth(), config.monitors.pancake_swap).await?;
        Ok(Self {
            streaming: Web3::new(streaming),
            requesting,
            buffer_size: config.core.buffer_size,
            monitor: Arc::new(pancake),
        })
    }

    async fn subscribe_pending_transactions(
        &self,
    ) -> anyhow::Result<impl Stream<Item = Transaction> + '_> {
        Ok(self
            .streaming
            .eth_subscribe()
            .subscribe_new_pending_transactions()
            .await
            .with_context(|| "failed to subscribe to new pending transactions")?
            .filter_map(|r| future::ready(r.ok()))
            .map({
                let eth = self.requesting.eth();
                move |h| eth.transaction(TransactionId::Hash(h))
            })
            .buffer_unordered(self.buffer_size)
            .filter_map(|r| future::ready(r.unwrap_or(None))))
    }
}

impl<ST, RT> App<ST, RT>
where
    ST: DuplexTransport + Send + Sync + 'static,
    <ST as DuplexTransport>::NotificationStream: Send,
    <ST as Transport>::Out: Send,
    RT: Transport + Send + Sync + 'static,
    <RT as Transport>::Out: Send,
{
    pub async fn run(self: Arc<Self>) -> anyhow::Result<()> {
        let mut tx_hashes = self.subscribe_pending_transactions().await?.boxed();

        println!("run");

        while let Some(tx) = tx_hashes.next().await {
            println!("{:#x}", tx.hash);
            // let s = self.clone();
            tokio::spawn(self.monitor.clone().process(tx));
        }

        Ok(())
    }

    async fn process(self: Arc<Self>, tx: Transaction) -> anyhow::Result<()> {
        self.monitor.clone().process(tx).await.map_err(Into::into)
    }
}
