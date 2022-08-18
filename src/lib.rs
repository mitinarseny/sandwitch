#![feature(iterator_try_collect)]
mod contracts;
mod pairs;
// mod pancake_swap;

use anyhow::Context;
use futures::TryFutureExt;
use serde::Deserialize;
use std::future;
use web3::contract::Options;
use web3::ethabi::Uint;

use futures::stream::{StreamExt, TryStreamExt};
use web3::transports::WebSocket;
use web3::types::{Address, Transaction, TransactionId, H160};
use web3::Web3;

use pairs::UnorderedPairs;

use self::contracts::pancake_swap::{
    RouterV2SwapExactETHForTokensInputs, PAIR, ROUTER_V2_SWAP_EXACT_ETH_FOR_TOKENS,
};
use self::pairs::UnorderedPair;
// use pancake_swap::SWAP_EXACT_ETH_FOR_TOKENS;

// use self::pancake_swap::{SwapExactETHForTokensInputs, ADDRESS as PANCAKE_SWAP_ROUTER_V2_ADDRESS};

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
        let pancake_swap_router_v2 = web3::contract::Contract::new(
            self.web3.eth(),
            contracts::pancake_swap::ROUTER_V2_ADDRESS,
            contracts::pancake_swap::ROUTER_V2.clone(),
        );
        let pancake_swap_factory_v2 = web3::contract::Contract::new(
            self.web3.eth(),
            contracts::pancake_swap::FACTORY_V2_ADDRESS,
            contracts::pancake_swap::FACTORY_V2.clone(),
        );

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
                        .map_or(false, |h| h == contracts::pancake_swap::ROUTER_V2_ADDRESS),
                )
            })
            .filter(|tx: &Transaction| {
                future::ready(
                    tx.input
                        .0
                        .starts_with(&ROUTER_V2_SWAP_EXACT_ETH_FOR_TOKENS.short_signature()),
                )
            })
            .filter_map(|tx: Transaction| {
                future::ready(
                    RouterV2SwapExactETHForTokensInputs::try_from(&tx.input.0[4..])
                        .map(move |i| (tx, i))
                        .ok(),
                )
            })
            .filter_map(
                |(tx, inputs): (Transaction, RouterV2SwapExactETHForTokensInputs)| {
                    future::ready({
                        let pairs = inputs
                            .path
                            .windows(2)
                            .map(|p| UnorderedPair(p[0], p[1]))
                            .filter(|p| self.pancake_config.token_pairs.contains(p))
                            .collect::<Vec<_>>();
                        if !pairs.is_empty() {
                            Some((tx, pairs))
                        } else {
                            None
                        }
                    })
                },
            )
            .flat_map_unordered(
                None,
                |(tx, pairs): (Transaction, Vec<UnorderedPair<H160>>)| {
                    futures::stream::iter(pairs)
                        .then(|p| {
                            pancake_swap_factory_v2.query::<(Address,), _, _, _>(
                                "getPair",
                                p.into_inner(),
                                None,
                                Options::default(),
                                None,
                            )
                        })
                        .filter_map(|r| future::ready(r.ok()))
                        .map(move |(a,)| (tx.clone(), a))
                        .boxed()
                },
            )
            .filter_map(|(tx, c)| {
                let pair = web3::contract::Contract::new(self.web3.eth(), c, PAIR.clone());
                async move {
                    pair.query::<(Uint, Uint, Uint), _, _, _>(
                        "getReserves",
                        (),
                        None,
                        Options::default(),
                        None,
                    )
                    .await
                    .map(move |r| (tx, r))
                    .ok()
                }
            })
            .boxed();
        while let Some((tx, pairs)) = tx_stream.next().await {
            println!("{:?}: {:?}", tx.hash, pairs);
        }

        Ok(())
    }
}
