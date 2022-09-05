use futures::stream::BoxStream;
use futures::{StreamExt, TryFutureExt};
use metrics::{register_counter, Counter};
use serde::Deserialize;
use std::collections::HashMap;
use std::future;
use tracing::{info, warn};
use web3::api::Eth;
use web3::contract::tokens::Detokenize;
use web3::types::{Address, Transaction, H256, U256};
use web3::Transport;

mod contracts;
mod factory;
mod pair;
mod router;
mod token;

use crate::timed::Timed;

use self::pair::Pair;
use self::router::Router;

use super::Monitor;

#[derive(Deserialize, Debug)]
pub struct PancakeSwapConfig {
    pub router: Address,
    pub bnb_limit: f64,
    pub gas_price: f64,
    pub gas_limit: f64,
    pub token_pairs: Vec<(Address, Address)>,
}

pub struct PancakeSwap<T: Transport> {
    router: Router<T>,
    bnb_limit: f64,
    gas_price: f64,
    gas_limit: f64,
    pair_contracts: HashMap<(Address, Address), Pair<T>>,
    metrics: Metrics,
}

struct Metrics {
    to_router: Counter,
    swap_exact_eth_for_tokens: Counter,
}

impl Metrics {
    fn new() -> Self {
        Self {
            to_router: register_counter!("sandwitch_pancake_swap_to_router"),
            swap_exact_eth_for_tokens: register_counter!(
                "sandwitch_pancake_swap_swap_exact_eth_for_tokens"
            ),
        }
    }
}

impl<T: Transport> PancakeSwap<T> {
    #[tracing::instrument(skip_all)]
    pub async fn from_config(eth: Eth<T>, config: PancakeSwapConfig) -> anyhow::Result<Self> {
        let router = Router::new(eth.clone(), config.router).await?;
        let factory = router.factory();
        info!(router = ?router.address(), factory = ?factory.address());

        info!(
            total = config.token_pairs.len(),
            "collecting pair contracts..."
        );
        let pair_contracts: HashMap<(Address, Address), Pair<T>> =
            futures::stream::iter(config.token_pairs)
                .map(move |p| Pair::new(eth.clone(), factory, p).map_ok(move |pair| (p, pair)))
                .buffer_unordered(50)
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

        Ok(Self {
            router,
            bnb_limit: config.bnb_limit,
            gas_price: config.gas_price,
            gas_limit: config.gas_limit,
            pair_contracts,
            metrics: Metrics::new(),
        })
    }
}

impl<T: Transport> PancakeSwap<T> {
    fn check_swap_exact_eth_for_tokens(&self, tx: &Transaction) -> bool {
        !tx.value.is_zero()
            && [tx.value, tx.gas].iter().all(|v| v.bits() <= 128) // for U256::as_u128
            && tx.to.map_or(false, |h| h == self.router.address())
            && tx.input.0.starts_with(
                &contracts::SWAP_EXACT_ETH_FOR_TOKENS.short_signature(), // TODO:
                                                                         // calculate signature before
            )
    }

    fn decode_swap_exact_eth_for_tokens_input(
        tx: &Transaction,
    ) -> Option<(u128, Vec<Address>, Address, U256)> {
        contracts::SWAP_EXACT_ETH_FOR_TOKENS
            .decode_input(&tx.input.0[4..])
            .ok()
            .map(<(U256, Vec<Address>, Address, U256)>::from_tokens)
            .map(Result::ok)
            .flatten()
            .filter(|(amount_out_min, ..)| amount_out_min.bits() <= 128)
            .map(|(amount_out_min, path, to, deadline)| {
                (amount_out_min.low_u128(), path, to, deadline)
            })
    }

    fn decode_swap_exact_eth_for_tokens_input2(
        tx: &Transaction,
    ) -> Option<(u128, (Address, Address))> {
        Self::decode_swap_exact_eth_for_tokens_input(tx)
            .filter(|(_, path, ..)| path.len() == 2)
            .map(|(amount_out_min, path, ..)| (amount_out_min, (path[0], path[1])))
    }

    async fn filter_map(&self, tx: Timed<Transaction>) -> Option<Timed<Swap<'_, T>>> {
        if !self.check_swap_exact_eth_for_tokens(&tx) {
            return None;
        }

        let (amount_out_min, (t0, t1)) = Self::decode_swap_exact_eth_for_tokens_input2(&tx)?;

        let pair = self.pair_contracts.get(&(t0, t1))?;
        let (t0, t1) = pair.tokens();

        let sw = Swap {
            tx_hash: tx.hash,
            gas: tx.gas.as_u128(),
            gas_price: t0.as_decimals(tx.gas_price?.low_u128()),
            amount_in: t0.as_decimals(tx.value.low_u128()),
            amount_out_min: t1.as_decimals(amount_out_min),
            reserves: pair.get_reserves().await.ok().map(|(r0, r1, _)| (r0, r1))?,
            pair,
        };

        Some(tx.map(|_| sw))
    }

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

struct Swap<'a, T: Transport> {
    tx_hash: H256,
    gas: u128,
    gas_price: f64,
    amount_in: f64,
    amount_out_min: f64,
    reserves: (f64, f64),
    pair: &'a Pair<T>,
}

impl<T> Monitor<Timed<Transaction>> for PancakeSwap<T>
where
    T: Transport + Send + Sync + 'static,
    <T as Transport>::Out: Send,
{
    type Output = Transaction;

    fn process<'a>(
        &'a mut self,
        stream: BoxStream<'a, Timed<Transaction>>,
    ) -> BoxStream<'a, Self::Output> {
        stream
            .map(|tx| self.filter_map(tx))
            .buffer_unordered(10)
            .filter_map(future::ready)
            .inspect(|sw| {
                if let Some((we_buy, our_min_b)) = self.calculate_amounts_in_and_out_min(
                    sw.reserves.0,
                    sw.reserves.1,
                    sw.amount_in,
                    sw.amount_out_min,
                    self.bnb_limit,
                ) {
                    let (t0, t1) = sw.pair.tokens();
                    println!(
                        "{:#x}, {}, {}, {}, {}, {:#x}, {:#x}, {}, {}, {:#x}, {}, {}, {}",
                        sw.tx_hash,
                        sw.gas,
                        sw.gas_price,
                        sw.amount_in,
                        sw.amount_out_min,
                        t0.address(),
                        t1.address(),
                        sw.reserves.0,
                        sw.reserves.1,
                        sw.pair.address(),
                        we_buy,
                        our_min_b,
                        sw.unix().as_millis()
                    );
                }
            })
            .filter_map(|_| future::ready(None))
            .boxed()
    }
}

fn calculate_max_amount(pool_a: f64, pool_b: f64, his_value_a: f64, his_value_min_b: f64) -> f64 {
    0.5 * (((his_value_a * (4.0 * pool_a * pool_b + his_value_a * his_value_min_b))
        / his_value_min_b)
        .sqrt()
        - 2.0 * pool_a
        - his_value_a)
}
