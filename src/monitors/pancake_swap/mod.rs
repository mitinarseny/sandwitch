use futures::future::BoxFuture;
use futures::stream::{BoxStream, FuturesUnordered};
use futures::{FutureExt, Stream, StreamExt, TryFutureExt, TryStreamExt};
use serde::Deserialize;
use std::collections::HashMap;
use std::future;
use std::sync::Arc;
use std::time::Duration;
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

        // let pair_contracts = tokio_stream::StreamExt::throttle(
        //     futures::stream::iter(config.token_pairs),
        //     Duration::from_millis(500),
        // )
        let pair_contracts = config
            .token_pairs
            .into_iter()
            .map(move |(t0, t1)| {
                Pair::new(eth.clone(), factory, (t0, t1)).map_ok(move |pair| ((t0, t1), pair))
            })
            .collect::<FuturesUnordered<_>>()
            .inspect_ok(|(_, pair)| println!("{pair}"))
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

    fn process(self: Arc<Self>, tx: Transaction) -> BoxFuture<'static, Result<(), Self::Error>> {
        async move {
            if !self.check_swap_exact_eth_for_tokens(&tx) {
                return Ok(());
            }

            let (amount_out_min, path, _, deadline) =
                match Self::decode_swap_exact_eth_for_tokens_input(&tx) {
                    Some(v) => v,
                    None => return Ok(()),
                };

            let (t0, t1) = if path.len() == 2 {
                (path[0], path[1])
            } else {
                return Ok(());
            };

            let pair = match self.pair_contracts.get(&(t0, t1)) {
                Some(v) => v,
                None => return Ok(()),
            };

            let (r0, r1, _) = pair.get_reserves().await?;
            let (amount_in, amount_out_min) = (
                pair.tokens().0.as_decimals(types::big_num::U256(tx.value)),
                pair.tokens()
                    .1
                    .as_decimals(types::big_num::U256(amount_out_min)),
            );

            println!(
                "{:#x}, {amount_in}, {amount_out_min}, ({t0:#x}, {t1:#x}), ({r0}, {r1}), {deadline}",
                tx.hash
            );

            Ok(())
        }
        .boxed()
    }
}

// impl<T> Monitor<Transaction> for PancakeSwap<T>
// where
//     T: Transport + Send + Sync + 'static,
//     <T as Transport>::Out: Send,
// {
//     type Error = web3::contract::Error;
//
//     fn process(
//         &self: Box<Self>,
//         stream: BoxStream<'static, Transaction>,
//     ) -> BoxFuture<'_, Result<(), Self::Error>> {
//         let mut s = stream
//             .filter({
//                 let router = self.router.clone();
//                 move |tx| {
//                     future::ready(
//                         !tx.value.is_zero()
//                             && tx.to.map_or(false, |h| h == router.address())
//                             && tx.input.0.starts_with(
//                                 &contracts::SWAP_EXACT_ETH_FOR_TOKENS.short_signature(), // TODO:
//                                                                                          // calculate signature before
//                             ),
//                     )
//                 }
//             })
//             .filter_map(|tx| {
//                 future::ready(
//                     contracts::SWAP_EXACT_ETH_FOR_TOKENS
//                         .decode_input(&tx.input.0[4..])
//                         .ok()
//                         .map(<(U256, Vec<Address>, Address, U256)>::from_tokens)
//                         .map(Result::ok)
//                         .flatten()
//                         .map(|(amount_out_min, path, _, deadline)| {
//                             (tx, (amount_out_min, path, deadline))
//                         }),
//                 )
//             })
//             // filter only path[2] swaps
//             .filter_map(|(tx, (amount_out_min, path, deadline))| {
//                 future::ready(
//                     (path.len() == 2)
//                         .then(move || (tx, (amount_out_min, (path[0], path[1]), deadline))),
//                 )
//             })
//             .filter_map({
//                 |(tx, (amount_out_min, (t0, t1), deadline))| async move {
//                     (*self).pair_contracts
//                         .get(&(t0, t1))
//                         .cloned()
//                         .map(move |pair| {
//                             let amount_in =
//                                 pair.tokens().0.as_decimals(types::big_num::U256(tx.value));
//                             let amount_out_min = pair
//                                 .tokens()
//                                 .1
//                                 .as_decimals(types::big_num::U256(amount_out_min));
//                             (tx, (amount_in, amount_out_min, (t0, t1), pair, deadline))
//                         })
//                 }
//             })
//             .filter_map(
//                 |(tx, (amount_in, amount_out_min, (t0, t1), pair, deadline))| {
//                     pair.get_reserves()
//                         .map_ok(move |(r0, r1, _)| {
//                             (
//                                 tx,
//                                 (amount_in, amount_out_min, (t0, t1), (r0, r1), deadline),
//                             )
//                         })
//                         .map(Result::ok)
//                 },
//             )
//             .boxed();
//
//         async move {
//             while let Some((tx, (amount_in, amount_out_min, (t0, t1), (r0, r1), deadline))) =
//                 s.next().await
//             {
//                 println!(
//                 "{:#x}, {amount_in}, {amount_out_min}, ({t0:?}, {t1:?}), ({r0:?}, {r1:?}), {deadline}",
//                 tx.hash
//             );
//             }
//             println!("end");
//             Ok(())
//         }.boxed()
//     }
// }

// impl<T> Monitor<Transaction> for PancakeSwap<T>
// where
//     T: Transport + Sync + Send + 'static,
//     <T as web3::Transport>::Out: Send,
// {
//     type Error = web3::contract::Error;
//
//     fn process(&mut self, tx: &Transaction) -> BoxFuture<'_, Result<(), Self::Error>> {
//         Box::pin(async move {
//             if !self.check_swap_exact_eth_for_tokens(tx) {
//                 return Ok(());
//             }
//
//             let (amount_out_min, path, _, deadline) =
//                 match Self::decode_swap_exact_eth_for_tokens_input(tx) {
//                     Some(v) => v,
//                     None => return Ok(()),
//                 };
//
//             let (t0, t1) = if path.len() == 2 {
//                 (path[0], path[1])
//             } else {
//                 return Ok(());
//             };
//
//             let pair = match self.pair_contracts.get(&(t0, t1)) {
//                 Some(v) => v,
//                 None => return Ok(()),
//             };
//
//             let (r0, r1) = pair.get_reserves().await?;
//
//             let (amount_in, amount_out_min) = (
//                 pair.tokens().0.as_decimals(types::big_num::U256(tx.value)),
//                 pair.tokens()
//                     .1
//                     .as_decimals(types::big_num::U256(amount_out_min)),
//             );
//
//             println!(
//                 "{:#x}, {amount_in}, {amount_out_min}, ({t0:#x}, {t1:#x}), ({r0}, {r1})",
//                 tx.hash
//             );
//
//             Ok(())
//         })
//     }
// }

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
