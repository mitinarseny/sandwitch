use std::collections::HashMap;
use std::sync::Arc;

use ethers::prelude::*;
use futures::lock::Mutex;
use futures::{
    future, future::BoxFuture, stream::FuturesUnordered, FutureExt, StreamExt, TryFutureExt,
};
use hex::ToHex;
use metrics::{describe_counter, register_counter, Counter, Unit};
use serde::Deserialize;
use tracing::{info, warn};

mod pair;
mod router;
mod token;

use self::pair::Pair;
use self::router::Router;

use crate::cached::Aption;
use crate::contracts::pancake_router_v2::SwapExactETHForTokensCall;

use super::{BlockMonitor, FunctionCallMonitor, TxMonitor};

#[derive(Deserialize, Debug)]
pub struct PancakeSwapConfig {
    pub router: Address,
    pub bnb_limit: f64,
    pub gas_price: f64,
    pub gas_limit: f64,
    pub token_pairs: Vec<(Address, Address)>,
}

pub struct PancakeSwap<M: Middleware> {
    client: Arc<M>,
    router: Router<M>,
    bnb_limit: f64,
    gas_price: f64,
    gas_limit: f64,
    pair_contracts: HashMap<(Address, Address), Mutex<Aption<Arc<Pair<M>>>>>,
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

        Ok(Self {
            bnb_limit: config.bnb_limit,
            gas_price: config.gas_price,
            gas_limit: config.gas_limit,
            pair_contracts: config
                .token_pairs
                .into_iter()
                .map(|p| (p, Mutex::new(None.into())))
                .collect(),
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
            client,
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

struct Swap {
    tx_hash: H256,
    gas: u128,
    gas_price: f64,
    amount_in: f64,
    amount_out_min: f64,
    reserves: (f64, f64),
}

impl<M> TxMonitor for PancakeSwap<M>
where
    M: Middleware + 'static,
{
    #[tracing::instrument(skip_all, fields(?tx.hash))]
    fn on_tx<'a>(&'a self, tx: &'a Transaction) -> BoxFuture<'a, anyhow::Result<Vec<Transaction>>> {
        if !tx.to.map_or(false, |h| h == self.router.address()) {
            None
        } else {
            self.metrics.to_router.increment(1);

            match *tx.input {
                [127, 243, 106, 181, ..] => {
                    <Self as FunctionCallMonitor<SwapExactETHForTokensCall>>::on_func_raw(self, tx)
                }
                _ => None,
            }
        }
        .unwrap_or_else(|| future::ok(Vec::new()).boxed())
    }
}

impl<M> BlockMonitor for PancakeSwap<M>
where
    M: Middleware,
{
    #[tracing::instrument(skip_all, fields(?block.hash))]
    fn on_block<'a>(&'a mut self, block: &'a Block<TxHash>) -> BoxFuture<'a, anyhow::Result<()>> {
        if block.hash.is_none() {
            return future::ok(()).boxed();
        }
        self.pair_contracts
            .iter_mut()
            .filter_map(|(_, p)| p.get_mut().as_mut())
            .map(|p| p.on_block())
            .collect::<FuturesUnordered<_>>()
            .collect::<()>()
            .map(Result::Ok)
            .boxed()
    }
}

impl<M> FunctionCallMonitor<SwapExactETHForTokensCall> for PancakeSwap<M>
where
    M: Middleware + 'static,
{
    fn on_func<'a>(
        &'a self,
        tx: &'a Transaction,
        inputs: SwapExactETHForTokensCall,
    ) -> BoxFuture<'a, anyhow::Result<Vec<Transaction>>> {
        async move {
            self.metrics.swap_exact_eth_for_tokens.increment(1);
            if inputs.path.len() != 2 {
                return Ok(Vec::new());
            }
            self.metrics.swap_exact_eth_for_tokens2.increment(1);
            let (t0, t1) = (inputs.path[0], inputs.path[1]);

            let pair = match self.pair_contracts.get(&(t0, t1)) {
                None => return Ok(Vec::new()),
                Some(pair) => pair
                    .lock()
                    .await
                    .get_or_try_insert_with(|| {
                        Pair::new(self.client.clone(), self.router.factory(), (t0, t1))
                            .map_ok(Arc::new)
                    })
                    .await?
                    .clone(),
            };
            // TODO: remove this pair if error is this contract does not exist no more

            pair.hit();
            let (t0, t1) = pair.tokens();

            let _sw = Swap {
                tx_hash: tx.hash,
                gas: tx.gas.as_u128(),
                gas_price: t0.as_decimals(tx.gas_price.unwrap().low_u128()),
                amount_in: t0.as_decimals(tx.value.low_u128()),
                amount_out_min: t1.as_decimals(inputs.amount_out_min.low_u128()),
                reserves: pair.reserves().map_ok(|(r0, r1, _)| (r0, r1)).await?,
            };
            drop(pair);

            info!(?tx.hash, "we got interesting transation");
            Ok(Vec::new())
        }
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
