use std::collections::HashMap;
use std::sync::Arc;

use ethers::{abi::AbiEncode, prelude::*};
use futures::lock::Mutex;
use futures::{
    future, future::BoxFuture, stream::FuturesUnordered, FutureExt, StreamExt, TryFutureExt,
};
use metrics::{describe_counter, register_counter, Counter, Unit};
use serde::Deserialize;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use crate::cached::Cached;
use crate::contracts::pancake_router_v2::{
    SwapETHForExactTokensCall, SwapExactETHForTokensCall, SwapExactTokensForETHCall,
};

mod pair;
mod router;
mod token;

use self::pair::Pair;
use self::router::Router;

use super::{FunctionCallMonitor, TxMonitor};
use crate::accounts::Accounts;

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
    pair_contracts: HashMap<(Address, Address), Cached<Arc<Pair<M>>>>,
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
                .map(|p| (p, None.into()))
                .collect(),
            metrics: Metrics {
                to_router: {
                    let c = register_counter!(
                        "sandwitch_pancake_swap_to_router",
                        "address" => router.address().encode_hex(),
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
    type Ok = ();
    type Error = anyhow::Error;

    #[tracing::instrument(skip_all, fields(?tx.hash))]
    fn process_tx<'a, P: JsonRpcClient, S: Signer>(
        &'a self,
        tx: &'a Transaction,
        block_hash: H256,
        accounts: &'a Accounts<P, S>,
    ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
        if !tx.to.map_or(false, |h| h == self.router.address()) {
            None
        } else {
            self.metrics.to_router.increment(1);

            match *tx.input {
                [127, 243, 106, 181, ..] => {
                    FunctionCallMonitor::<SwapExactETHForTokensCall>::maybe_process_func_raw(
                        self, tx, block_hash, accounts,
                    )
                }
                _ => None,
            }
        }
        .unwrap_or_else(|| future::ok(()).boxed())
    }
}

impl<'a, M> FunctionCallMonitor<'a, SwapExactETHForTokensCall> for PancakeSwap<M>
where
    M: Middleware + 'static,
{
    type Ok = ();
    type Error = anyhow::Error;

    fn process_func<P: JsonRpcClient, S: Signer>(
        &'a self,
        tx: &'a Transaction,
        block_hash: H256,
        inputs: SwapExactETHForTokensCall,
        accounts: &'a Accounts<P, S>,
    ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
        async move {
            self.metrics.swap_exact_eth_for_tokens.increment(1);
            let (t0_address, t1_address) = match inputs.path[..] {
                [t0, t1] => (t0, t1),
                _ => return Ok(()),
            };
            self.metrics.swap_exact_eth_for_tokens2.increment(1);

            let pair = match self.pair_contracts.get(&(t0_address, t1_address)) {
                None => return Ok(()),
                Some(pair) => {
                    pair.get_or_try_insert_with(|| {
                        Pair::new(
                            self.client.clone(),
                            self.router.factory(),
                            (t0_address, t1_address),
                        )
                        .map_ok(Arc::new)
                    })
                    .await?
                }
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
                reserves: pair
                    .reserves(block_hash)
                    .await
                    .map(|(r0, r1, _)| (r0, r1))?,
            };
            // let s1 = SwapETHForExactTokensCall {
            //     amount_out: todo!(),
            //     path: todo!(),
            //     to: todo!(),
            //     deadline: todo!(),
            // };

            info!(?tx.hash, "we got interesting transation");
            // Ok([
            //     TransactionRequest::new()
            //         .to(self.router.address())
            //         .gas_price(tx.gas_price.unwrap() + 1)
            //         .data(
            //             SwapETHForExactTokensCall {
            //                 amount_out: todo!(),
            //                 path: [t0_address, t1_address].into(),
            //                 to: todo!(),
            //                 deadline: todo!(),
            //             }
            //             .encode(),
            //         ),
            //     TransactionRequest::new()
            //         .to(self.router.address())
            //         .gas_price(tx.gas_price.unwrap() - 1)
            //         .data(
            //             SwapExactTokensForETHCall {
            //                 amount_in: todo!(),
            //                 amount_out_min: todo!(),
            //                 path: [t1_address, t0_address].into(),
            //                 to: todo!(),
            //                 deadline: todo!(),
            //             }
            //             .encode(),
            //         ),
            // ]
            // .into())

            Ok(())
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
