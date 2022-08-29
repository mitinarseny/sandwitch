#![feature(
    iterator_try_collect,
    result_option_inspect,
    future_join,
    result_flattening
)]
mod monitors;
mod types;

use std::future;

use anyhow::Context;
use serde::Deserialize;

use monitors::Monitor;

use futures::stream::{StreamExt, TryStreamExt};
use web3::transports::{Http, WebSocket};
use web3::types::{Transaction, TransactionId};
use web3::{DuplexTransport, Transport, Web3};

use self::monitors::MultiMonitor;
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
    monitors: Box<MultiMonitor<Transaction, web3::contract::Error>>,
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
    RT: Transport + Send + Sync + 'static,
{
    async fn from_transports(streaming: ST, requesting: RT, config: Config) -> anyhow::Result<Self>
    where
        <RT as Transport>::Out: Send,
    {
        let requesting = Web3::new(requesting);
        let pancake =
            PancakeSwap::from_config(requesting.eth(), config.monitors.pancake_swap).await?;
        Ok(Self {
            streaming: Web3::new(streaming),
            requesting,
            buffer_size: config.core.buffer_size,
            monitors: Box::new(MultiMonitor::new(
                config.core.buffer_size,
                vec![Box::new(pancake)],
            )),
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
        let tx_hashes = self
            .streaming
            .eth_subscribe()
            .subscribe_new_pending_transactions()
            .await
            .with_context(|| "failed to subscribe to new pending transactions")?
            // .inspect(|tx| println!("{:?}", tx))
            .map_ok({
                let eth = self.requesting.eth();
                move |h| eth.transaction(TransactionId::Hash(h))
            })
            .try_buffer_unordered(self.buffer_size)
            .filter_map(|r| future::ready(r.unwrap_or(None)))
            // .inspect(|tx| println!("{:#?}", tx))
            .boxed();

        dbg!("run");

        self.monitors.process(tx_hashes).await.map_err(Into::into)
    }
}
