use futures::future::BoxFuture;
use futures::stream::{BoxStream, FuturesUnordered};
use futures::{FutureExt, StreamExt, TryFutureExt, TryStreamExt};
use num::rational::Ratio;
use num::{BigUint, ToPrimitive};
use serde::Deserialize;
use std::collections::HashMap;
use std::future;
use web3::api::Eth;
use web3::contract::tokens::Detokenize;
use web3::types::{Address, Transaction, U256};
use web3::Transport;

mod contracts;
mod factory;
mod pair;
mod router;
mod token;

use self::pair::Pair;
use self::router::Router;

use super::Monitor;

#[derive(Deserialize, Debug)]
pub struct PancakeSwapConfig {
    pub router: Address,
    pub token_pairs: Vec<(Address, Address)>,
}

pub struct PancakeSwap<T: Transport> {
    router: Router<T>,
    pair_contracts: HashMap<(Address, Address), Pair<T>>,
}

impl<T: Transport> PancakeSwap<T> {
    pub async fn from_config(
        eth: Eth<T>,
        config: PancakeSwapConfig,
    ) -> web3::contract::Result<Self> {
        let router = Router::new(eth.clone(), config.router).await?;
        let factory = router.factory();

        let pair_contracts = config
            .token_pairs
            .into_iter()
            .map(move |(t0, t1)| {
                Pair::new(eth.clone(), factory, (t0, t1)).map_ok(move |pair| ((t0, t1), pair))
            })
            .collect::<FuturesUnordered<_>>()
            // .inspect_ok(|(_, pair)| println!("{pair}"))
            .try_collect()
            .await?;

        dbg!("finish");

        Ok(Self {
            router,
            pair_contracts,
        })
    }
}

impl<T: Transport> PancakeSwap<T> {
    fn check_swap_exact_eth_for_tokens(&self, tx: &Transaction) -> bool {
        !tx.value.is_zero()
            && tx.value.bits() <= 128 // for U256::as_u128
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
}

impl<T> Monitor<Transaction> for PancakeSwap<T>
where
    T: Transport + Send + Sync + 'static,
    <T as Transport>::Out: Send,
{
    type Error = web3::contract::Error;

    fn process<'a>(
        &'a mut self,
        stream: BoxStream<'a, Transaction>,
    ) -> BoxFuture<'a, Result<(), Self::Error>> {
        let mut stream = stream
            .filter(|tx| future::ready(self.check_swap_exact_eth_for_tokens(tx)))
            .filter_map(|tx| {
                future::ready(
                    Self::decode_swap_exact_eth_for_tokens_input2(&tx)
                        .map(move |inputs| (tx, inputs)),
                )
            })
            .filter_map(|(tx, (amount_out_min, (t0, t1)))| {
                future::ready(
                    self.pair_contracts
                        .get(&(t0, t1))
                        .map(|pair| (tx, amount_out_min, pair)),
                )
            })
            .map(|(tx, amount_out_min, pair)| {
                pair.get_reserves().map_ok(move |(r0, r1, _)| {
                    let (t0, t1) = pair.tokens();
                    let amount_in = t0.as_decimals(tx.value.low_u128());
                    let gas = tx.gas.low_u128();
                    let gas_price = tx.gas_price.map(|p| t0.as_decimals(p.low_u128()));
                    let amount_out_min = t1.as_decimals(amount_out_min);
                    (
                        tx,
                        gas,
                        gas_price,
                        amount_in,
                        amount_out_min,
                        (t0, t1),
                        (r0, r1),
                        pair,
                    )
                })
            })
            .buffer_unordered(10)
            .filter_map(|r| future::ready(r.ok()))
            .boxed();

        async move {
            while let Some((tx, gas, gas_price, amount_in, amount_out_min, (t0, t1), (r0, r1), pair)) =
                stream.next().await
            {
                dbg!("{:#x}", tx.hash);
                println!(
                    "{:#x}, {gas}, {}, {amount_in}, {amount_out_min}, {:#x}, {:#x}, {r0}, {r1}, {:#x}",
                    tx.hash,
                    gas_price.unwrap_or(0.0),
                    t0.address(),
                    t1.address(),
                    pair.address(),
                );
            }
            Ok(())
        }
        .boxed()
    }
}

fn calculate_max_amount<A, B, C, D>(
    pool_a: A,
    pool_b: B,
    his_value_a: C,
    his_value_min_b: D,
) -> Ratio<BigUint>
where
    A: Into<Ratio<BigUint>>,
    B: Into<Ratio<BigUint>>,
    C: Into<Ratio<BigUint>>,
    D: Into<Ratio<BigUint>>,
{
    let (pool_a, pool_b, his_value_a, his_value_min_b) = (
        pool_a.into(),
        pool_b.into(),
        his_value_a.into(),
        his_value_min_b.into(),
    );
    ((his_value_a.clone()
        * (pool_a * pool_b * BigUint::from(4u8) + his_value_a * his_value_min_b.clone()))
        / his_value_min_b)
}
