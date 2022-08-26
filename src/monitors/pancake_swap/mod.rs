use futures::future::BoxFuture;
use futures::stream::BoxStream;
use futures::{FutureExt, Stream, StreamExt, TryFutureExt, TryStreamExt};
use itertools::Itertools;
use serde::Deserialize;
use std::collections::HashMap;
use std::future;
use web3::contract::tokens::Detokenize;
use web3::contract::Options;
use web3::types::{Address, Transaction, U256};
use web3::Transport;

use super::Monitor;

mod contracts;

pub struct PancakeSwap<T: Transport> {
    web3: web3::Web3<T>,
    router_v2: web3::contract::Contract<T>,
    pair_contracts: HashMap<(Address, Address), Address>,
    token_decimals: HashMap<Address, u8>,
}

#[derive(Deserialize, Debug)]
pub struct PancakeSwapConfig {
    pub token_pairs: Vec<(Address, Address)>,
}

impl<T: Transport> PancakeSwap<T> {
    pub async fn from_config(
        web3: web3::Web3<T>,
        config: PancakeSwapConfig,
    ) -> web3::contract::Result<Self> {
        let router_v2 = web3::contract::Contract::new(
            web3.eth(),
            contracts::router_v2::ADDRESS,
            contracts::router_v2::ROUTER_V2.clone(),
        );
        let factory_v2 = web3::contract::Contract::new(
            web3.eth(),
            contracts::factory_v2::ADDRESS,
            contracts::factory_v2::FACTORY_V2.clone(),
        );
        Ok(Self {
            web3: web3.clone(),
            router_v2,
            pair_contracts: futures::stream::iter(config.token_pairs.iter().cloned())
                .map(|p| {
                    factory_v2
                        .query::<(Address,), _, _, _>("getPair", p, None, Options::default(), None)
                        .map_ok(move |(a,)| (p, a))
                })
                .buffered(10) // TODO
                .try_collect()
                .await?,
            token_decimals: futures::stream::iter(
                config
                    .token_pairs
                    .into_iter()
                    .flat_map(|(t0, t1)| [t0, t1].into_iter()),
            )
            .map(|t| {
                let token =
                    web3::contract::Contract::new(web3.eth(), t, contracts::token::TOKEN.clone());
                async move {
                    token
                        .query::<(u8,), _, _, _>("decimals", (), None, Options::default(), None)
                        .map_ok(move |(c,)| (t, c))
                        .await
                }
            })
            .buffered(10) // TODO
            .try_collect()
            .await?,
        })
    }
}

impl<T> PancakeSwap<T>
where
    T: Transport + Sync + Send,
{
    async fn process_transactions<S>(self, txs: S)
    where
        S: Stream<Item = Transaction> + Unpin + Send,
        <T as web3::Transport>::Out: Send,
    {
        let mut s = txs
            .filter(|tx| {
                future::ready(
                    !tx.value.is_zero()
                        && tx.to.map_or(false, |h| h == self.router_v2.address())
                        && tx.input.0.starts_with(
                            &contracts::router_v2::SWAP_EXACT_ETH_FOR_TOKENS.short_signature(), // TODO:
                                                                                                // calculate signature before
                        ),
                )
            })
            .filter_map(|tx| {
                future::ready(
                    contracts::router_v2::SWAP_EXACT_ETH_FOR_TOKENS
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
            .flat_map_unordered(None, |(tx, (amount_out_min, path, ..))| {
                futures::stream::iter(
                    path.into_iter()
                        .tuple_windows()
                        .filter_map(|p| self.pair_contracts.get(&p).cloned().map(|pair| (p, pair))),
                )
                .filter_map(|((t0, t1), pair)| {
                    self.get_reserves(pair)
                        .map_ok(move |(mut r0, mut r1, _)| {
                            if t1 < t0 {
                                // sort by hash value
                                (r0, r1) = (r1, r0);
                            }
                            ((t0, t1), (r0, r1), pair)
                        })
                        .map(|r| r.ok())
                })
                .map(move |token_pools| (tx.clone(), amount_out_min, token_pools))
                .boxed()
            })
            .boxed();
        while let Some((tx, amount_out_min, token_pools)) = s.next().await {
            println!("{:#x}, {amount_out_min:?}, {token_pools:?}", tx.hash);
        }
    }

    async fn get_reserves(&self, pair: Address) -> web3::contract::Result<(u128, u128, u32)> {
        let pair =
            web3::contract::Contract::new(self.web3.eth(), pair, contracts::pair::PAIR.clone());
        pair.query::<(u128, u128, u32), _, _, _>("getReserves", (), None, Options::default(), None)
            .await
    }
}

impl<T> Monitor for PancakeSwap<T>
where
    T: Transport + Sync + Send + 'static,
    <T as web3::Transport>::Out: Send,
{
    type Item = Transaction;

    fn process(self: Box<Self>, stream: BoxStream<'_, Self::Item>) -> BoxFuture<'_, ()> {
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
