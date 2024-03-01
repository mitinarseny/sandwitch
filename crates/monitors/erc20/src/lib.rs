#![feature(is_some_and)]

mod pool;

use std::collections::HashMap;

use async_trait::async_trait;
use ethers::{
    abi::{AbiDecode, RawLog},
    contract::EthEvent,
    providers::Middleware,
    types::{Address, H256, U256},
};
use futures::try_join;
use tracing::instrument;

// use contracts::{
//     pancake_swap::i_pancake_router_02::{
//         IPancakeRouter02Calls, SwapETHForExactTokensCall, SwapExactETHForTokensCall,
//         SwapExactTokensForETHCall, SwapExactTokensForTokensCall, SwapTokensForExactETHCall,
//         SwapTokensForExactTokensCall,
//     },
//     pancake_toaster::{
//         i_pancake_pair::IPancakePairCalls,
//         pancake_toaster::{FrontRunSwapExtCall, PancakeToaster, PancakeToasterCalls},
//     },
// };

// use crate::{
//     accounts::Accounts,
//     cached::Cached,
//     monitors::{
//         inputs::{FromWithTx, IntoWithTx},
//         ContractCallMonitor, ContractCallMonitorExt, TxMonitor,
//     },
//     timeout::TimeoutProvider,
// };

// mod pair;
// mod router;
// mod token;
//
// use self::{pair::Pair, router::Router};

mod monitor;

use sandwitch_contracts::{
    erc20::ApproveCall,
    multicall::{Call, ContractCall},
    pancake_swap::{pair::SyncFilter, router::PancakeRouterCalls},
    pancake_toaster::{
        BackRunSwapAllCall, FrontRunSwapExtCall, FrontRunSwapExtReturn, PancakeToaster,
    },
};

use sandwitch_engine::{
    block::{PendingBlock, TxWithLogs},
    monitor::BlockMonitor,
    transactions::Transaction,
};

pub(crate) struct PancakeSwapMonitor<M: Middleware> {
    router: Address,
    toaster: PancakeToaster<M>,
    base_token: Address,
}

#[async_trait]
impl<M: Middleware + 'static> BlockMonitor<M> for PancakeSwapMonitor<M> {
    #[instrument(skip_all)]
    async fn process_pending_block(&self, block: &PendingBlock<M>) -> anyhow::Result<()> {
        for adjacent_txs in block.iter_adjacent_txs() {
            // let tokens_out_deltas = HashMap::new();
            let token_out_deltas = self.token_out_deltas(&adjacent_txs)?;
            for (token_out, (reserve_before, reserve_after)) in token_out_deltas {}
            let front_run = block.first_in_block_candidate((
                ContractCall::new(
                    self.base_token,
                    ApproveCall {
                        spender: self.toaster.address(),
                        amount: U256::MAX, // TODO: cache already approved tokens
                    },
                )
                .must(),
                ContractCall::new(
                    self.toaster.address(),
                    FrontRunSwapExtCall {
                        from: block.account(),
                        amount_in: swap.amount_in,
                        amount_out: swap.amount_out,
                        eth_in: swap.eth_in,
                        path: swap.path,
                        index_in: index_in.into(),
                    },
                )
                .must(),
                // TODO: add in backwards direction
            ));
            let back_run = adjacent_txs.back_run_candidate((
                ContractCall::new(
                    token_out,
                    ApproveCall {
                        spender: self.toaster.address(),
                        amount: U256::MAX, // TODO
                    },
                )
                .must(),
                ContractCall::new(
                    self.toaster.address(),
                    BackRunSwapAllCall {
                        token_in,
                        token_out,
                    },
                )
                .must(),
            ));

            let (
                    Ok((Ok(_approved), Ok(FrontRunSwapExtReturn {
                        his_amount_in,
                        his_amount_out,
                        our_amount_in: front_run_amount_in,
                        our_amount_out: front_run_amount_out,
                        new_reserve_in: reserve_in_after_front_run,
                        new_reserve_out: reserve_out_after_front_run,
                    }))),
                    Ok(front_run_cost),
                    Ok(back_run_cost),
                ) = try_join!(
                    front_run.call(),
                    front_run.estimate_fee(),
                    back_run.estimate_fee(),
                )? else {
                    // someone reverted
                    continue;
                };

            // TODO: prifit calculations

            let reserve_in_after_patty = reserve_in_after_front_run + his_amount_in;
            let reserve_out_after_patty = reserve_out_after_front_run - his_amount_out;

            let back_run_amount_out = get_amount_out(
                front_run_amount_out,
                reserve_in_after_patty,
                reserve_out_after_patty,
            );

            let delta = back_run_amount_out - front_run_amount_in;
            let cost = front_run_cost + back_run_cost;
            if delta > cost {
                block.add_to_send([front_run.into(), back_run.into()]).await;
            }
        }
        Ok(())
    }
}

impl<M: Middleware + 'static> PancakeSwapMonitor<M> {
    fn token_out_deltas(
        &self,
        txs: &[TxWithLogs],
    ) -> anyhow::Result<HashMap<Address, (Reserves, Reserves)>> {
        let mut reserves = HashMap::new();
        for tx in txs {
            if !tx.to.is_some_and(|to| to == self.router) {
                continue;
            }
            // TODO: check all logs and extract deltas
            let Some(swap) = Swap::from_tx(&tx) else {
                continue;
            };
            // TODO: check that base_token is included only once
            let Some(index_in) = swap.path.iter().position(|token| *token == self.base_token) else {
                continue;
            };
            if index_in == swap.path.len() - 1 {
                // we need base token to be not the last one
                continue;
            }
            // TODO: check if all tokens are in whitelist?

            let token_out = swap.path[index_in + 1]; // TODO
            let pair = pair_address(self.base_token, token_out);

            let sync_topic_0: H256 = SyncFilter::signature();

            let sync_log = tx
                .logs
                .iter()
                .rfind(|l| {
                    l.address == pair && l.topics.first().is_some_and(|t| *t == sync_topic_0)
                })
                .unwrap()
                .clone();
            let sync = SyncFilter::decode_log(&RawLog {
                topics: sync_log.topics,
                data: sync_log.data.to_vec(),
            })?;
            reserves.entry(token_out).or_insert(todo!());
        }
        Ok(reserves)
    }
}

fn pair_address(token0: Address, token1: Address) -> Address {
    todo!()
}



const BIP: u64 = 1_0000;

fn get_amount_out(amount_in: U256, reserve_in: U256, reserve_out: U256) -> U256 {
    (reserve_out * amount_in) / (reserve_in * BIP + amount_in)
}


// struct DecodedTransaction<'a, C: EthCall> {
//     tx: &'a Transaction,
//     inputs: C,
// }

// impl<'a, C: EthCall> TryFrom<&'a Transaction> for DecodedTransaction<'a, C> {
//     type Error = AbiError;

//     fn try_from(tx: &'a Transaction) -> Result<Self, Self::Error> {
//         Ok(Self {
//             inputs: C::decode(&tx.input)?,
//             tx,
//         })
//     }
// }

// impl<M: Middleware> PendingTxsMonitor for PancakeSwapMonitor<M> {
//     type Error = anyhow::Error;

//     fn process_pending_tx<'a>(
//         &'a self,
//         txs: &'a Transaction,
//         parent_block_hash: H256,
//     ) -> BoxFuture<'a, Result<Vec<Transaction>, Self::Error>> {
//         txs.into_iter()
//             .filter(|tx| tx.to.map_or(false, |to| to == self.router))
//             .filter_map(|tx| DecodedTransaction::<IPancakeRouter02Calls>::try_from(tx).ok())
//             .filter_map(|tx| {
//                 let (amount_in, amount_out, eth_in, path) = match tx.inputs {
//                     IPancakeRouter02Calls::SwapExactETHForTokens(s) => {
//                         (tx.tx.value, s.amount_out_min, true, s.path)
//                     }
//                     IPancakeRouter02Calls::SwapETHForExactTokens(s) => {
//                         (tx.tx.value, s.amount_out, true, s.path)
//                     }
//                     IPancakeRouter02Calls::SwapExactTokensForTokens(s) => {
//                         (s.amount_in, s.amount_out_min, false, s.path)
//                     }
//                     IPancakeRouter02Calls::SwapTokensForExactTokens(s) => {
//                         (s.amount_in_max, s.amount_out, false, s.path)
//                     }
//                     IPancakeRouter02Calls::SwapExactTokensForETH(s) => {
//                         (s.amount_in, s.amount_out_min, false, s.path)
//                     }
//                     IPancakeRouter02Calls::SwapTokensForExactETH(s) => {
//                         (s.amount_in_max, s.amount_out, false, s.path)
//                     }
//                     _ => return None,
//                 };
//                 // TODO: check that all (expect for last if not playable) tokens are in whitelist
//                 // check that nobody else touches exploited pairs
//             })
//             .map(|(amount_in)| {
//                 self.toaster
//                     .front_run_swap_ext(
//                         tx.from,
//                         amount_in,
//                         amount_out,
//                         eth_in,
//                         path,
//                         index_in,
//                         parent_block_hash,
//                     )
//                     .block(BlockNumber::Pending)
//                     .gas_price(gas_price)
//                     .call()
//             })
//             .collect::<FuturesUnordered<_>>()
//             .try_next()
//             .await
//     }
// }

// // * filter by router addr
// // * decode inputs
// // * try decode params from inputs
// // * find base_token and ensure all in whitelist (except for last)
// // * check that nobody else previously exploited any pair in path (except for in other direction)
// // * front_run_swap_ext() on every inputs
// // * calculate profits using initial tx
// // * use first profitable or compare using now_or_never()
// // * send front_run_swap() and back_run_swap()
// impl<M: Middleware> PancakeSwapMonitor<M> {
//     fn process_tx() -> Option<()> {}
// }

// #[derive(Deserialize, Debug)]
// pub struct PancakeSwapConfig {
//     pub router: Address,
//     pub bnb_limit: f64,
//     pub gas_price: f64,
//     pub gas_limit: f64,
//     pub token_pairs: Vec<(Address, Address)>,
// }
//
// pub(crate) struct PancakeSwap<P: JsonRpcClient, S: Signer> {
//     client: Arc<Provider<P>>,
//     accounts: Arc<Accounts<P, S>>,
//     router: Router<P>,
//     bnb_limit: f64,
//     gas_price: f64,
//     gas_limit: f64,
//     pair_contracts: HashMap<(Address, Address), Cached<Arc<Pair<P>>>>,
//     metrics: Metrics,
// }
//
// struct Metrics {
//     to_router: Counter,
//     swap_exact_eth_for_tokens: Counter,
//     swap_exact_eth_for_tokens2: Counter,
// }
//
// impl<P, S> PancakeSwap<P, S>
// where
//     P: JsonRpcClient + 'static,
//     S: Signer,
// {
//     #[tracing::instrument(skip_all)]
//     pub(crate) async fn from_config(
//         client: impl Into<Arc<Provider<P>>>,
//         accounts: impl Into<Arc<Accounts<P, S>>>,
//         config: PancakeSwapConfig,
//     ) -> anyhow::Result<Self> {
//         let client: Arc<_> = client.into();
//         let router = Router::new(client.clone(), config.router).await?;
//         let factory = router.factory();
//         info!(router = ?router.address(), factory = ?factory.address());
//
//         Ok(Self {
//             accounts: accounts.into(),
//             bnb_limit: config.bnb_limit,
//             gas_price: config.gas_price,
//             gas_limit: config.gas_limit,
//             pair_contracts: config
//                 .token_pairs
//                 .into_iter()
//                 .map(|p| (p, None.into()))
//                 .collect(),
//             metrics: Metrics {
//                 to_router: {
//                     let c = register_counter!(
//                         "sandwitch_pancake_swap_to_router",
//                         "address" => format!("{:#x}", router.address()),
//                     );
//                     describe_counter!(
//                         "sandwitch_pancake_swap_to_router",
//                         Unit::Count,
//                         "TX to PancakeRouter"
//                     );
//                     c
//                 },
//                 swap_exact_eth_for_tokens: {
//                     let c = register_counter!("sandwitch_pancake_swap_swapExactETHForTokens");
//                     describe_counter!(
//                         "sandwitch_pancake_swap_swapExactETHForTokens",
//                         Unit::Count,
//                         "TX calling swapExactETHForTokens"
//                     );
//                     c
//                 },
//                 swap_exact_eth_for_tokens2: {
//                     let c = register_counter!("sandwitch_pancake_swap_swapExactETHForTokens2");
//                     describe_counter!(
//                         "sandwitch_pancake_swap_swapExactETHForTokens2",
//                         Unit::Count,
//                         "TX calling swapExactETHForTokens with only two tokens"
//                     );
//                     c
//                 },
//             },
//             router,
//             client,
//         })
//     }
// }
//
// impl<P, S> TxMonitor for PancakeSwap<P, S>
// where
//     P: JsonRpcClient,
//     S: Signer,
// {
//     type Ok = ();
//     type Error = anyhow::Error;
//
//     #[tracing::instrument(skip_all, fields(?tx.hash))]
//     fn process_tx<'a>(
//         &'a self,
//         tx: &'a Transaction,
//         block_hash: H256,
//     ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
//         <Self as ContractCallMonitorExt<IPancakeRouter02Calls>>::maybe_process_call(
//             self, tx, block_hash,
//         )
//         .unwrap_or_else(|| future::ok(()).boxed())
//     }
// }
//
// impl<'a, P, S> ContractCallMonitor<'a, IPancakeRouter02Calls> for PancakeSwap<P, S>
// where
//     P: JsonRpcClient,
//     S: Signer,
// {
//     type Ok = ();
//     type Error = anyhow::Error;
//
//     fn filter(&self, tx_to: Address) -> bool {
//         tx_to == self.router.address()
//     }
//
//     fn process_call(
//         &'a self,
//         tx: &'a Transaction,
//         block_hash: H256,
//         inputs: IPancakeRouter02Calls,
//     ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
//         match inputs {
//             IPancakeRouter02Calls::SwapExactETHForTokens(_)
//             | IPancakeRouter02Calls::SwapExactTokensForTokens(_)
//             | IPancakeRouter02Calls::SwapExactTokensForETH(_) => {
//                 <Self as ContractCallMonitor<SwapExactIn>>::process_call(
//                     self,
//                     tx,
//                     block_hash,
//                     match inputs {
//                         IPancakeRouter02Calls::SwapExactETHForTokens(c) => c.into_with_tx(tx),
//                         IPancakeRouter02Calls::SwapExactTokensForTokens(c) => c.into(),
//                         IPancakeRouter02Calls::SwapExactTokensForETH(c) => c.into(),
//                         _ => unreachable!(),
//                     },
//                 )
//             }
//             IPancakeRouter02Calls::SwapETHForExactTokens(_)
//             | IPancakeRouter02Calls::SwapTokensForExactTokens(_)
//             | IPancakeRouter02Calls::SwapTokensForExactETH(_) => {
//                 <Self as ContractCallMonitor<SwapExactOut>>::process_call(
//                     self,
//                     tx,
//                     block_hash,
//                     match inputs {
//                         IPancakeRouter02Calls::SwapETHForExactTokens(c) => c.into_with_tx(tx),
//                         IPancakeRouter02Calls::SwapTokensForExactTokens(c) => c.into(),
//                         IPancakeRouter02Calls::SwapTokensForExactETH(c) => c.into(),
//                         _ => unreachable!(),
//                     },
//                 )
//             }
//             _ => future::ok(()).boxed(),
//         }
//     }
// }
//
// impl<'a, P: JsonRpcClient, S: Signer> ContractCallMonitor<'a, SwapExactIn> for PancakeSwap<P, S> {
//     type Ok = ();
//     type Error = anyhow::Error;
//
//     fn process_call(
//         &'a self,
//         tx: &'a Transaction,
//         block_hash: H256,
//         inputs: SwapExactIn,
//     ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
//         for &[t0, t1] in inputs.path.array_windows::<2>() {}
//     }
// }
//
// impl<'a, P: JsonRpcClient, S: Signer> ContractCallMonitor<'a, SwapExactOut> for PancakeSwap<P, S> {
//     type Ok = ();
//     type Error = anyhow::Error;
//
//     fn process_call(
//         &'a self,
//         tx: &'a Transaction,
//         block_hash: H256,
//         inputs: SwapExactOut,
//     ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
//         todo!()
//     }
// }
//
// impl<P: JsonRpcClient, S: Signer> PancakeSwap<P, S> {}

// impl<'a, P, S> FunctionCallMonitor<'a, SwapExactETHForTokensCall> for PancakeSwap<P, S>
// where
//     P: JsonRpcClient + 'static,
//     S: Signer,
// {
//     type Ok = ();
//     type Error = anyhow::Error;
//
//     fn process_func(
//         &'a self,
//         tx: &'a Transaction,
//         block_hash: H256,
//         inputs: SwapExactETHForTokensCall,
//     ) -> BoxFuture<'a, Result<Self::Ok, Self::Error>> {
//         async move {
//             self.metrics.swap_exact_eth_for_tokens.increment(1);
//             let (t0_address, t1_address) = match inputs.path[..] {
//                 [t0, t1] => (t0, t1),
//                 _ => return Ok(()),
//             };
//             self.metrics.swap_exact_eth_for_tokens2.increment(1);
//
//             let pair = match self.pair_contracts.get(&(t0_address, t1_address)) {
//                 None => return Ok(()),
//                 Some(pair) => {
//                     pair.get_or_try_insert_with(|| {
//                         Pair::new(
//                             self.client.clone(),
//                             self.router.factory(),
//                             (t0_address, t1_address),
//                         )
//                         .map_ok(Arc::new)
//                     })
//                     .await?
//                 }
//             };
//             // TODO: remove this pair if error is this contract does not exist no more
//
//             pair.hit();
//             let (t0, t1) = pair.tokens();
//
//             let _sw = Swap {
//                 tx_hash: tx.hash,
//                 gas: tx.gas.as_u128(),
//                 gas_price: t0.as_decimals(tx.gas_price.unwrap().low_u128()),
//                 amount_in: t0.as_decimals(tx.value.low_u128()),
//                 amount_out_min: t1.as_decimals(inputs.amount_out_min.low_u128()),
//                 reserves: pair
//                     .reserves(block_hash)
//                     .await
//                     .map(|(r0, r1, _)| (r0, r1))?,
//             };
//             // let s1 = SwapETHForExactTokensCall {
//             //     amount_out: todo!(),
//             //     path: todo!(),
//             //     to: todo!(),
//             //     deadline: todo!(),
//             // };
//
//             info!(?tx.hash, "we got interesting transation");
//             // Ok([
//             //     TransactionRequest::new()
//             //         .to(self.router.address())
//             //         .gas_price(tx.gas_price.unwrap() + 1)
//             //         .data(
//             //             SwapETHForExactTokensCall {
//             //                 amount_out: todo!(),
//             //                 path: [t0_address, t1_address].into(),
//             //                 to: todo!(),
//             //                 deadline: todo!(),
//             //             }
//             //             .encode(),
//             //         ),
//             //     TransactionRequest::new()
//             //         .to(self.router.address())
//             //         .gas_price(tx.gas_price.unwrap() - 1)
//             //         .data(
//             //             SwapExactTokensForETHCall {
//             //                 amount_in: todo!(),
//             //                 amount_out_min: todo!(),
//             //                 path: [t1_address, t0_address].into(),
//             //                 to: todo!(),
//             //                 deadline: todo!(),
//             //             }
//             //             .encode(),
//             //         ),
//             // ]
//             // .into())
//
//             Ok(())
//         }
//         .boxed()
//     }
// }
//
//

// struct SwapExactIn {
//     amount_in: U256,
//     amount_out_min: U256,
//     path: Vec<Address>,
//     deadline: U256,
// }
//
// impl FromWithTx<SwapExactETHForTokensCall> for SwapExactIn {
//     fn from_with_tx(value: SwapExactETHForTokensCall, tx: &Transaction) -> Self {
//         let SwapExactETHForTokensCall {
//             amount_out_min,
//             path,
//             deadline,
//             ..
//         } = value;
//         Self {
//             amount_in: tx.value,
//             amount_out_min,
//             path,
//             deadline,
//         }
//     }
// }
//
// impl From<SwapExactTokensForTokensCall> for SwapExactIn {
//     fn from(value: SwapExactTokensForTokensCall) -> Self {
//         let SwapExactTokensForTokensCall {
//             amount_in,
//             amount_out_min,
//             path,
//             deadline,
//             ..
//         } = value;
//         Self {
//             amount_in,
//             amount_out_min,
//             path,
//             deadline,
//         }
//     }
// }
//
// impl From<SwapExactTokensForETHCall> for SwapExactIn {
//     fn from(value: SwapExactTokensForETHCall) -> Self {
//         let SwapExactTokensForETHCall {
//             amount_in,
//             amount_out_min,
//             path,
//             deadline,
//             ..
//         } = value;
//         Self {
//             amount_in,
//             amount_out_min,
//             path,
//             deadline,
//         }
//     }
// }
//
// struct SwapExactOut {
//     amount_in_max: U256,
//     amount_out: U256,
//     path: Vec<Address>,
//     deadline: U256,
// }
//
// impl FromWithTx<SwapETHForExactTokensCall> for SwapExactOut {
//     fn from_with_tx(value: SwapETHForExactTokensCall, tx: &Transaction) -> Self {
//         let SwapETHForExactTokensCall {
//             amount_out,
//             path,
//             deadline,
//             ..
//         } = value;
//         Self {
//             amount_in_max: tx.value,
//             amount_out,
//             path,
//             deadline,
//         }
//     }
// }
//
// impl From<SwapTokensForExactTokensCall> for SwapExactOut {
//     fn from(value: SwapTokensForExactTokensCall) -> Self {
//         let SwapTokensForExactTokensCall {
//             amount_in_max,
//             amount_out,
//             path,
//             deadline,
//             ..
//         } = value;
//         Self {
//             amount_in_max,
//             amount_out,
//             path,
//             deadline,
//         }
//     }
// }
//
// impl From<SwapTokensForExactETHCall> for SwapExactOut {
//     fn from(value: SwapTokensForExactETHCall) -> Self {
//         let SwapTokensForExactETHCall {
//             amount_in_max,
//             amount_out,
//             path,
//             deadline,
//             ..
//         } = value;
//         Self {
//             amount_in_max,
//             amount_out,
//             path,
//             deadline,
//         }
//     }
// }
