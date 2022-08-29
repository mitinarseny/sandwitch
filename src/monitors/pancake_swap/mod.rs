use futures::future::BoxFuture;
use futures::stream::{BoxStream, FuturesUnordered};
use futures::{FutureExt, StreamExt, TryFutureExt, TryStreamExt};
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
use super::super::types;

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

        println!("finish");

        Ok(Self {
            router,
            pair_contracts,
        })
    }
}

impl<T: Transport> PancakeSwap<T> {
    fn check_swap_exact_eth_for_tokens(&self, tx: &Transaction) -> bool {
        !tx.value.is_zero()
            && tx.to.map_or(false, |h| h == self.router.address())
            && tx.input.0.starts_with(
                &contracts::SWAP_EXACT_ETH_FOR_TOKENS.short_signature(), // TODO:
                                                                         // calculate signature before
            )
    }

    fn decode_swap_exact_eth_for_tokens_input(
        tx: &Transaction,
    ) -> Option<(U256, Vec<Address>, Address, U256)> {
        contracts::SWAP_EXACT_ETH_FOR_TOKENS
            .decode_input(&tx.input.0[4..])
            .ok()
            .map(<(U256, Vec<Address>, Address, U256)>::from_tokens)
            .map(Result::ok)
            .flatten()
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
                    Self::decode_swap_exact_eth_for_tokens_input(&tx)
                        .filter(|(_, path, ..)| path.len() == 2)
                        .map(|(amount_out_min, path, ..)| {
                            (tx, (amount_out_min, (path[0], path[1])))
                        }),
                )
            })
            .filter_map(|(tx, (amount_out_min, (t0, t1)))| {
                future::ready(
                    self.pair_contracts
                        .get(&(t0, t1))
                        .map(|pair| (tx, amount_out_min, (t0, t1), pair)),
                )
            })
            .map(|(tx, amount_out_min, (t0, t1), pair)| {
                pair.get_reserves()
                    .map_ok(move |(r0, r1, _)| (tx, amount_out_min, (t0, t1), (r0, r1), pair))
            })
            .buffer_unordered(10)
            .filter_map(|r| future::ready(r.ok()))
            .map(|(tx, amount_out_min, (t0, t1), (r0, r1), pair)| {
                let (amount_in, amount_out_min) = (
                    pair.tokens().0.as_decimals(types::big_num::U256(tx.value)),
                    pair.tokens()
                        .1
                        .as_decimals(types::big_num::U256(amount_out_min)),
                );
                (tx, (amount_in, amount_out_min, (t0, t1), (r0, r1)))
            })
            .boxed();

        async move {
            while let Some((tx, (amount_in, amount_out_min, (t0, t1), (r0, r1)))) =
                stream.next().await
            {
                println!(
                    "{:#x}, {amount_in}, {amount_out_min}, ({t0:#x}, {t1:#x}), ({r0}, {r1})",
                    tx.hash
                );
            }
            Ok(())
        }
        .boxed()
    }
}
