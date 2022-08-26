#![feature(iterator_try_collect, result_option_inspect)]
mod monitors;

use anyhow::Context;
use futures::{FutureExt, TryFutureExt};
use serde::Deserialize;
use std::future;

use futures::stream::{Stream, StreamExt};
use web3::transports::WebSocket;
use web3::types::{Transaction, TransactionId};
use web3::{DuplexTransport, Transport, Web3};

use self::monitors::pancake_swap::{PancakeSwap, PancakeSwapConfig};
use self::monitors::{Monitor, MultiMonitor};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub url: String,

    #[serde(default)]
    pub buffer_size: usize,

    pub monitors: MonitorsConfig,
}

#[derive(Deserialize, Debug)]
pub struct MonitorsConfig {
    pub pancake_swap: PancakeSwapConfig,
}

pub struct App<T>
where
    T: DuplexTransport,
{
    web3: Web3<T>,
    buffer_size: usize,
    monitors: Box<MultiMonitor<Transaction>>,
}

impl App<WebSocket> {
    pub async fn from_config(config: Config) -> anyhow::Result<Self> {
        let transport = web3::transports::WebSocket::new(&config.url).await?;
        Self::from_transport(transport, config).await
    }
}

impl<T> App<T>
where
    T: DuplexTransport + Send + Sync + 'static,
    <T as DuplexTransport>::NotificationStream: Send,
    <T as Transport>::Out: Send,
{
    async fn from_transport(transport: T, config: Config) -> anyhow::Result<Self> {
        let web3 = web3::Web3::new(transport.clone());

        Ok(Self {
            web3: web3.clone(),
            buffer_size: config.buffer_size,
            monitors: MultiMonitor::new(vec![Box::new(
                PancakeSwap::from_config(web3, config.monitors.pancake_swap).await?,
            )]),
        })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let txs = self.subscribe_pending_transactions().await?;
        println!("sibscribed new pending transactions");
        self.monitors.process(Box::pin(txs)).await;
        Ok(())
    }

    async fn subscribe_pending_transactions(
        &self,
    ) -> anyhow::Result<impl Stream<Item = Transaction>> {
        self.web3
            .eth_subscribe()
            .subscribe_new_pending_transactions()
            .map(|r| r.with_context(|| "failed to subscribe to new pending transactions"))
            .map_ok(|s| {
                s.filter_map(|r| future::ready(r.inspect_err(|e| println!("{e:?}")).ok()))
                    // TODO: filter unique tx hashes
                    .map({
                        let eth = self.web3.eth();
                        move |h| eth.transaction(TransactionId::Hash(h))
                    })
                    .buffered(self.buffer_size)
                    .filter_map(|r| future::ready(r.unwrap_or(None)))
            })
            .await
    }
}
