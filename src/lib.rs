#![feature(iterator_try_collect)]
mod pairs;
mod pancake_swap;

use anyhow::Context;
use serde::Deserialize;
use std::future;

use futures::StreamExt;
use web3::transports::WebSocket;
use web3::types::{Address, Transaction, TransactionId};
use web3::Web3;

use pairs::UnorderedPairs;
use pancake_swap::SWAP_EXACT_ETH_FOR_TOKENS;

use self::pancake_swap::{SwapExactETHForTokensInputs, ADDRESS as PANCAKE_SWAP_ROUTER_V2_ADDRESS};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub url: String,

    #[serde(default)]
    pub buffer_size: usize,

    pub pancake_swap: PancakeSwapConfig,
}

#[derive(Deserialize, Debug)]
pub struct PancakeSwapConfig {
    pub token_pairs: UnorderedPairs<Address>,
}

pub struct App {
    web3: Web3<WebSocket>,
    buffer_size: usize,
    pancake_config: PancakeSwapConfig,
}

impl App {
    pub async fn from_config(config: Config) -> web3::Result<Self> {
        let transport = web3::transports::WebSocket::new(&config.url).await?;
        let web3 = web3::Web3::new(transport);

        Ok(Self {
            web3,
            buffer_size: config.buffer_size,
            pancake_config: config.pancake_swap,
        })
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let mut tx_stream = self
            .web3
            .eth_subscribe()
            .subscribe_new_pending_transactions()
            .await
            .with_context(|| "failed to subscribe to new pending transactions")?
            .filter_map(|r| future::ready(r.ok()))
            .map({
                let eth = self.web3.eth();
                move |h| eth.transaction(TransactionId::Hash(h))
            })
            .buffered(self.buffer_size)
            .filter_map(|r| future::ready(r.unwrap_or(None)))
            .filter(|tx: &Transaction| {
                future::ready(
                    tx.to
                        .map_or(false, |h| h == *PANCAKE_SWAP_ROUTER_V2_ADDRESS),
                )
            })
            .filter(|tx: &Transaction| {
                future::ready(
                    tx.input
                        .0
                        .starts_with(&SWAP_EXACT_ETH_FOR_TOKENS.short_signature()),
                )
            })
            .filter_map(|tx: Transaction| {
                future::ready(
                    SwapExactETHForTokensInputs::try_from(&tx.input.0[4..])
                        .map(move |i| (tx, i))
                        .ok(),
                )
            })
            .filter(|(_, SwapExactETHForTokensInputs { path, .. })| {
                future::ready({
                    path.windows(2).any(|p| {
                        self.pancake_config
                            .token_pairs
                            .contains(&(p[0], p[1]).into())
                    })
                })
            });
        while let Some((tx, inputs)) = tx_stream.next().await {
            println!("{:?}: {:?}", tx.hash, inputs.path);
        }

        Ok(())
    }
}
