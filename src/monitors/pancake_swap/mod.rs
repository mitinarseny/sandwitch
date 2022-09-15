use ethers::abi::AbiDecode;
use ethers::prelude::*;
use futures::future::BoxFuture;
use futures::{FutureExt, StreamExt, TryFutureExt};
use hex::ToHex;
use metrics::{describe_counter, increment_counter, register_counter, Counter, Unit};
use serde::Deserialize;
use std::collections::HashMap;
use std::future;
use std::sync::Arc;
use tracing::{info, warn};

mod pair;
mod router;
mod token;

use self::pair::{CachedPair, Pair};
use self::router::Router;

use crate::contracts::pancake_router_v2::SwapExactETHForTokensCall;

use super::{Monitor, TxMonitor};

#[derive(Deserialize, Debug)]
pub struct PancakeSwapConfig {
    pub router: Address,
    pub bnb_limit: f64,
    pub gas_price: f64,
    pub gas_limit: f64,
    pub token_pairs: Vec<(Address, Address)>,
}

pub struct PancakeSwap<M: Middleware> {
    router: Router<M>,
    bnb_limit: f64,
    gas_price: f64,
    gas_limit: f64,
    pair_contracts: HashMap<(Address, Address), CachedPair<M>>,
    metrics: Metrics,
}

struct Metrics {
    to_router: Counter,
    swap_exact_eth_for_tokens: Counter,
    swap_exact_eth_for_tokens2: Counter,
}

impl<M> PancakeSwap<M>
where
    M: Middleware,
{
    #[tracing::instrument(skip_all)]
    pub async fn from_config(client: Arc<M>, config: PancakeSwapConfig) -> anyhow::Result<Self>
    where
        M: Clone + 'static,
    {
        let router = Router::new(client.clone(), config.router).await?;
        let factory = router.factory();
        info!(router = ?router.address(), factory = ?factory.address());

        info!(
            total = config.token_pairs.len(),
            "collecting pair contracts..."
        );
        let pair_contracts: HashMap<(Address, Address), CachedPair<M>> =
            futures::stream::iter(config.token_pairs)
                .map(move |p| {
                    Pair::new(client.clone(), factory, p).map_ok(move |pair| (p, pair.into()))
                })
                .buffer_unordered(5)
                .filter_map(|r| {
                    future::ready(
                        r.inspect_err(|err| {
                            warn!(%err, "failed to initialize pair, skipping...");
                        })
                        .ok(),
                    )
                })
                .collect()
                .await;
        info!(collected = pair_contracts.len(), "collected pair contracts");

        register_counter!("sandwitch_pancake_swap_swapExactETHForTokens2_filtered");
        Ok(Self {
            bnb_limit: config.bnb_limit,
            gas_price: config.gas_price,
            gas_limit: config.gas_limit,
            pair_contracts,
            metrics: Metrics {
                to_router: {
                    let c = register_counter!(
                        "sandwitch_pancake_swap_to_router",
                        "address" => router.address().encode_hex::<String>(),
                    );
                    describe_counter!(
                        "sandwitch_pancake_swap_to_router",
                        Unit::Count,
                        "TX to PancakeRouter"
                    );
                    c
                },
                swap_exact_eth_for_tokens: {
                    let c = register_counter!("sandwitch_pancake_swap_swapExactETHForTokens");
                    describe_counter!(
                        "sandwitch_pancake_swap_swapExactETHForTokens",
                        Unit::Count,
                        "TX calling swapExactETHForTokens"
                    );
                    c
                },
                swap_exact_eth_for_tokens2: {
                    let c = register_counter!("sandwitch_pancake_swap_swapExactETHForTokens2");
                    describe_counter!(
                        "sandwitch_pancake_swap_swapExactETHForTokens2",
                        Unit::Count,
                        "TX calling swapExactETHForTokens with only two tokens"
                    );
                    c
                },
            },
            router,
        })
    }
}

impl<M: Middleware> PancakeSwap<M> {
    fn tx_fee(&self) -> f64 {
        self.gas_price * self.gas_limit
    }

    fn calculate_amounts_in_and_out_min(
        &self,
        pool_a: f64,
        pool_b: f64,
        his_value_a: f64,
        his_value_min_b: f64,
        our_limit_a: f64,
    ) -> Option<(f64, f64)> {
        let we_buy = calculate_max_amount(pool_a, pool_b, his_value_a, his_value_min_b);
        if we_buy <= 0.0 {
            return None;
        }
        let we_buy = we_buy.min(our_limit_a);
        let we_get = (pool_b * we_buy) / (pool_a + we_buy);
        let (g1, g2) = (self.tx_fee(), self.tx_fee()); // TODO: gas price
        let our_min_b =
            (g1 + g2 + we_buy) * (pool_b - his_value_min_b) / (his_value_a + pool_a + we_buy);
        if we_get <= our_min_b {
            return None;
        }
        Some((we_buy, our_min_b))
    }
}

struct Swap<'a, M: Middleware> {
    tx_hash: H256,
    gas: u128,
    gas_price: f64,
    amount_in: f64,
    amount_out_min: f64,
    reserves: (f64, f64),
    pair: &'a CachedPair<M>,
}

impl<M> Monitor<Transaction> for PancakeSwap<M>
where
    M: Middleware,
{
    fn process(&mut self, tx: Transaction) -> BoxFuture<'_, ()> {
        async move {
            if tx.value.is_zero() || tx.to.map_or(false, |h| h == self.router.address()) {
                return;
            }

            self.metrics.to_router.increment(1);

            let c = match SwapExactETHForTokensCall::decode(&tx.input.0)
                .ok()
                .inspect(|_| self.metrics.swap_exact_eth_for_tokens.increment(1))
                .filter(|c| c.path.len() == 2)
            {
                Some(v) => v,
                None => return,
            };
            self.metrics.swap_exact_eth_for_tokens2.increment(1);

            let pair = match self.pair_contracts.get_mut(&(c.path[0], c.path[1])) {
                Some(v) => v,
                None => return,
            };
            pair.hit();
            let (t0, t1) = pair.tokens();

            let sw = Swap {
                tx_hash: tx.hash,
                gas: tx.gas.as_u128(),
                gas_price: t0.as_decimals(
                    match tx.gas_price {
                        Some(v) => v,
                        None => return,
                    }
                    .low_u128(),
                ),
                amount_in: t0.as_decimals(tx.value.low_u128()),
                amount_out_min: t1.as_decimals(c.amount_out_min.low_u128()),
                reserves: match pair.get_reserves().await.ok().map(|(r0, r1, _)| (r0, r1)) {
                    Some(v) => v,
                    None => return,
                },
                pair,
            };
        }
        .boxed()
        // stream
        //     .filter(|tx| {
        //         future::ready(
        //             !tx.value.is_zero() && tx.to.map_or(false, |h| h == self.router.address()),
        //         )
        //     })
        //     .inspect(|_| self.metrics.to_router.increment(1))
        //     .map(|tx| async {
        //         let c = SwapExactETHForTokensCall::decode(&tx.input.0)
        //             .ok()
        //             .inspect(|_| self.metrics.swap_exact_eth_for_tokens.increment(1))
        //             .filter(|c| c.path.len() == 2)?;
        //         self.metrics.swap_exact_eth_for_tokens2.increment(1);
        //
        //         let pair = self.pair_contracts.get(&(c.path[0], c.path[1]))?;
        //         let (t0, t1) = pair.tokens();
        //
        //         let sw = Swap {
        //             tx_hash: tx.hash,
        //             gas: tx.gas.as_u128(),
        //             gas_price: t0.as_decimals(tx.gas_price?.low_u128()),
        //             amount_in: t0.as_decimals(tx.value.low_u128()),
        //             amount_out_min: t1.as_decimals(c.amount_out_min.low_u128()),
        //             reserves: pair.get_reserves().await.ok().map(|(r0, r1, _)| (r0, r1))?,
        //             pair,
        //         };
        //
        //         Some(tx.map(move |_| sw))
        //     })
        //     .buffer_unordered(10)
        //     .filter_map(future::ready)
        //     .inspect(|sw| {
        //         if let Some((we_buy, our_min_b)) = self.calculate_amounts_in_and_out_min(
        //             sw.reserves.0,
        //             sw.reserves.1,
        //             sw.amount_in,
        //             sw.amount_out_min,
        //             self.bnb_limit,
        //         ) {
        //             let (t0, t1) = sw.pair.tokens();
        //             println!(
        //                 "{:#x}, {}, {}, {}, {}, {:#x}, {:#x}, {}, {}, {:#x}, {}, {}, {}",
        //                 sw.tx_hash,
        //                 sw.gas,
        //                 sw.gas_price,
        //                 sw.amount_in,
        //                 sw.amount_out_min,
        //                 t0.address(),
        //                 t1.address(),
        //                 sw.reserves.0,
        //                 sw.reserves.1,
        //                 sw.pair.address(),
        //                 we_buy,
        //                 our_min_b,
        //                 sw.unix().as_millis()
        //             );
        //         }
        //     })
        //     .filter_map(|_| future::ready(None))
        //     .boxed()
    }
}

impl<M> Monitor<Block<TxHash>> for PancakeSwap<M>
where
    M: Middleware,
{
    fn process(&mut self, input: Block<TxHash>) -> BoxFuture<'_, ()> {
        for (_, p) in self.pair_contracts.iter_mut() {
            p.clear_cache();
        }
        future::ready(()).boxed()
    }
}

impl<M> TxMonitor for PancakeSwap<M>
where
    M: Middleware,
{
    fn flush(&mut self) -> Vec<Transaction> {
        Vec::new()
        // todo!()
    }
}

fn calculate_max_amount(pool_a: f64, pool_b: f64, his_value_a: f64, his_value_min_b: f64) -> f64 {
    0.5 * (((his_value_a * (4.0 * pool_a * pool_b + his_value_a * his_value_min_b))
        / his_value_min_b)
        .sqrt()
        - 2.0 * pool_a
        - his_value_a)
}
