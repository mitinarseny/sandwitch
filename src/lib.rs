#![feature(iterator_try_collect)]
mod contracts;
mod pairs;
// mod pancake_swap;

use anyhow::Context;
use futures::TryFutureExt;
use serde::Deserialize;
use std::collections::HashMap;
use std::future;
use web3::contract::tokens::Detokenize;
use web3::contract::Options;
use web3::ethabi::Uint;

use futures::stream::{StreamExt, TryStreamExt};
use web3::transports::WebSocket;
use web3::types::{Address, Transaction, TransactionId, H160, U256};
use web3::{DuplexTransport, Transport, Web3};

use pairs::UnorderedPairs;

use self::contracts::pancake_swap::{
    RouterV2SwapExactETHForTokensInputs, PAIR, ROUTER_V2_SWAP_EXACT_ETH_FOR_TOKENS,
};
use self::pairs::UnorderedPair;

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

pub struct App<T>
where
    T: DuplexTransport,
{
    web3: Web3<T>,
    pancake_swap_router_v2: web3::contract::Contract<T>,
    pancake_swap_factory_v2: web3::contract::Contract<T>,
    buffer_size: usize,
    pair_contracts: HashMap<UnorderedPair<Address>, Address>,
}

impl App<WebSocket> {
    pub async fn from_config(config: Config) -> anyhow::Result<Self> {
        let transport = web3::transports::WebSocket::new(&config.url).await?;
        Self::from_transport(transport, config).await
    }
}

impl<T> App<T>
where
    T: DuplexTransport + Send + Sync,
    <T as DuplexTransport>::NotificationStream: Send,
    <T as Transport>::Out: Send,
{
    async fn from_transport(transport: T, config: Config) -> anyhow::Result<Self> {
        let web3 = web3::Web3::new(transport);
        let eth = web3.eth();
        let pancake_swap_factory_v2 = web3::contract::Contract::new(
            eth.clone(),
            contracts::pancake_swap::FACTORY_V2_ADDRESS,
            contracts::pancake_swap::FACTORY_V2.clone(),
        );

        let pair_contracts = futures::stream::iter(config.pancake_swap.token_pairs)
            .then(|p| {
                pancake_swap_factory_v2
                    .query::<(Address,), _, _, _>(
                        "getPair",
                        p.clone().into_inner(),
                        None,
                        Options::default(),
                        None,
                    )
                    .map_ok(move |(a,)| (p, a))
            })
            .try_collect()
            .await?;

        Ok(Self {
            web3,
            pancake_swap_router_v2: web3::contract::Contract::new(
                eth,
                contracts::pancake_swap::ROUTER_V2_ADDRESS,
                contracts::pancake_swap::ROUTER_V2.clone(),
            ),
            pancake_swap_factory_v2,
            buffer_size: config.buffer_size,
            pair_contracts,
        })
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let mut tx_stream =
            self.web3
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
                .filter_map(|tx: Transaction| {
                    future::ready(
                        (tx.to
                            .map_or(false, |h| h == contracts::pancake_swap::ROUTER_V2_ADDRESS)
                            && tx.input.0.starts_with(
                                &ROUTER_V2_SWAP_EXACT_ETH_FOR_TOKENS.short_signature(),
                            ))
                        .then(|| {
                            ROUTER_V2_SWAP_EXACT_ETH_FOR_TOKENS
                                .decode_input(&tx.input.0[4..])
                                .ok()
                                .and_then(|v| {
                                    <(U256, Vec<Address>, Address, U256)>::from_tokens(v).ok()
                                })
                        })
                        .flatten()
                        .map(move |inputs| (tx, inputs)),
                    )
                })
                .flat_map(
                    |(tx, (_, path, ..)): (Transaction, (_, Vec<Address>, _, _))| {
                        futures::stream::iter(
                            path.windows(2)
                                .map(|p| UnorderedPair(p[0], p[1]))
                                .filter_map(|p| self.pair_contracts.get(&p).cloned())
                                .map(move |c| (tx.clone(), c))
                                .collect::<Vec<_>>(),
                        )
                    },
                )
                .inspect(|(tx, c)| println!("{:?}: {c:?}", tx.hash))
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
        while let Some((tx, reserves)) = tx_stream.next().await {
            println!("{:?}: {:?}", tx.hash, reserves);
        }

        Ok(())
    }
}
