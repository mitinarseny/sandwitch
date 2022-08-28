use futures::future::BoxFuture;
use futures::stream::BoxStream;
use futures::{FutureExt, Stream, StreamExt, TryFutureExt, TryStreamExt};
use serde::Deserialize;
use std::collections::HashMap;
use std::future;
use std::time::Duration;
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
    // tokens: HashMap<Address, Token<T>>,
}

impl<T: Transport> PancakeSwap<T> {
    pub async fn from_config(
        web3: web3::Web3<T>,
        config: PancakeSwapConfig,
    ) -> web3::contract::Result<Self> {
        let router = Router::new(web3.eth(), config.router).await?;
        let factory = router.factory();

        let pair_contracts = futures::stream::iter(config.token_pairs)
            .map(|ts| Pair::new(web3.eth(), factory, ts.clone()).map_ok(move |p| (ts, p)))
            .buffer_unordered(50) // TODO
            .inspect_ok(|pair| println!("{:?}", pair.0))
            .try_collect()
            .await?;

        Ok(Self {
            router,
            pair_contracts,
        })
    }
}

impl<T> PancakeSwap<T>
where
    T: Transport + Sync + Send,
{
    async fn process_transactions<S>(self, txs: S) -> Result<(), ()>
    where
        S: Stream<Item = Transaction> + Unpin + Send,
        <T as web3::Transport>::Out: Send,
    {
        let mut s = txs
            .filter(|tx| {
                future::ready(
                    !tx.value.is_zero()
                        && tx.to.map_or(false, |h| h == self.router.address())
                        && tx.input.0.starts_with(
                            &contracts::SWAP_EXACT_ETH_FOR_TOKENS.short_signature(), // TODO:
                                                                                     // calculate signature before
                        ),
                )
            })
            .filter_map(|tx| {
                future::ready(
                    contracts::SWAP_EXACT_ETH_FOR_TOKENS
                        .decode_input(&tx.input.0[4..])
                        .ok()
                        .map(<(U256, Vec<Address>, Address, U256)>::from_tokens)
                        .map(Result::ok)
                        .flatten()
                        .map(|(amount_out_min, path, _, deadline)| {
                            (tx, (amount_out_min, path, deadline))
                        }),
                )
            })
            // filter only path[2] swaps
            .filter_map(|(tx, (amount_out_min, path, deadline))| {
                future::ready(
                    (path.len() == 2)
                        .then(move || (tx, (amount_out_min, (path[0], path[1]), deadline))),
                )
            })
            .filter_map(|(tx, (amount_out_min, (t0, t1), deadline))| {
                future::ready(self.pair_contracts.get(&(t0, t1)).map(move |pair| {
                    let amount_in = pair.tokens().0.as_decimals(types::big_num::U256(tx.value));
                    let amount_out_min = pair
                        .tokens()
                        .1
                        .as_decimals(types::big_num::U256(amount_out_min));
                    (tx, (amount_in, amount_out_min, (t0, t1), pair, deadline))
                }))
            })
            .filter_map(
                |(tx, (amount_in, amount_out_min, (t0, t1), pair, deadline))| {
                    pair.get_reserves()
                        .map_ok(move |(r0, r1, _)| {
                            (
                                tx,
                                (amount_in, amount_out_min, (t0, t1), (r0, r1), deadline),
                            )
                        })
                        .map(Result::ok)
                },
            )
            .boxed();
        while let Some((tx, (amount_in, amount_out_min, (t0, t1), (r0, r1), deadline))) = s.next().await {
            println!(
                "{:#x}, {amount_in}, {amount_out_min}, ({t0:?}, {t1:?}), ({r0:?}, {r1:?}), {deadline}",
                tx.hash
            );
        }
        println!("end");
        Ok(())
    }
}

impl<T> Monitor for PancakeSwap<T>
where
    T: Transport + Sync + Send + 'static,
    <T as web3::Transport>::Out: Send,
{
    type Item = Transaction;
    type Error = ();

    fn process(
        self: Box<Self>,
        stream: BoxStream<'_, Self::Item>,
    ) -> BoxFuture<'_, Result<(), Self::Error>> {
        Box::pin((*self).process_transactions(stream))
    }
}

// fn extract_reserves<'a>(
//     &'a self,
//     stream: impl Stream<Item = (Transaction, (Uint, Vec<Address>, Uint))> + 'a,
// ) -> impl Stream<
//     Item = (
//         Transaction,
//         (Uint, (Address, Uint, Address, Address, Uint, u8), Uint),
//     ),
// > + 'a {
//     // TODO: check for cycles in path
//     stream.flat_map(|(tx, (amount_out_min, path, deadline))| {
//         futures::stream::iter(path.into_iter().tuple_windows().filter_map(|(t0, t1)| {
//             self.pair_contracts
//                 .get(&(t0, t1))
//                 .cloned()
//                 .map(|pair| (t0, t1, pair))
//         }))
//         .filter_map(|(t0, t1, (pair, decimals))| {
//             self.get_reserves(pair)
//                 .map_ok(move |(mut r0, mut r1, _)| {
//                     if t1 < t0 {
//                         (r0, r1) = (r1, r0);
//                     }
//                     (t0, r0, pair, t1, r1, decimals)
//                 })
//                 .map(|r| r.ok())
//         })
//         .map(move |token_pools| (tx.clone(), (amount_out_min, token_pools, deadline)))
//     })
// }
//
// async fn process_transactions(
//     &self,
//     txs: impl Stream<Item = Transaction> + Send,
// ) -> anyhow::Result<()> {
//     let txs_with_inputs = self.filter_pancake_swap_exact_eth_for_tokens(txs);
//     let mut tx_with_token_reserves = self.extract_reserves(txs_with_inputs).boxed();
//
//     while let Some((tx, (amount_out_min, (t0, r0, pair, t1, r1, decimals), _))) =
//         tx_with_token_reserves.next().await
//     {
//         // println!("{:?}", self.pre_max(r0, r1, tx.value, amount_out_min));
//         println!(
//             "{:#x}, {}, {}, {:#x}, {}, {:#x}, {:#x}, {}, {}",
//             tx.hash, tx.value, amount_out_min, t0, r0, pair, t1, r1, decimals
//         );
//     }
//     Ok(())
// }
//
// async fn get_reserves(&self, pair: Address) -> web3::contract::Result<(u128, u128, u32)> {
//     let pair = web3::contract::Contract::new(self.web3.eth(), pair, PAIR.clone());
//     pair.query::<(u128, u128, u32), _, _, _>("getReserves", (), None, Options::default(), None)
//         .await
// }
//
// fn pre_max(reserve0: u128, reserve1: u128, amount0: U256, amount_out_min: U256) -> U256 {
//     let amount0 = BigUint::from_bytes_le({
//         let buf: [u8; size_of::<U256>()];
//         amount0.to_little_endian(&mut buf);
//         &buf
//     });
//     let amount_out_min = BigUint::from_bytes_le({
//         let buf: [u8; size_of::<U256>()];
//         amount_out_min.to_little_endian(&mut buf);
//         &buf
//     });
//
//     U256(
//         ((Ratio::new(amount0, amount_out_min)
//             * (BigUint::from(4u8) * reserve0 * reserve1 + amount0 * amount_out_min))
//             .to_integer()
//             .sqrt()
//             - BigUint::from(2u8) * reserve1
//             - amount0)
//             .to_u64_digits()
//             .try_into()
//             .unwrap(),
//     )
// }
//
// async fn balance_of(&self, token: Address, address: Address) -> web3::contract::Result<Uint> {
//     let token_contract = web3::contract::Contract::new(self.web3.eth(), token, TOKEN.clone());
//     token_contract
//         .query::<(U256,), _, _, _>("balanceOf", (address,), None, Options::default(), None)
//         .map_ok(|(balance,)| balance)
//         .await
// }
//
