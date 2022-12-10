#![feature(
    future_join,
    iterator_try_collect,
    result_flattening,
    result_option_inspect,
    entry_insert,
    poll_ready
)]

pub(crate) mod abort;
pub(crate) mod accounts;
mod app;
pub(crate) mod cached;
pub(crate) mod contracts;
pub(crate) mod engine;
pub mod monitors;

pub use app::{App, Config, EngineConfig, MonitorsConfig};

// mod metriced;
// mod stream_utils;
// mod timed;
// pub mod with;

// use self::{
//     accounts::{Account, Accounts},
//     engine::{Engine, TopTxMonitor},
//     monitors::{
//         pancake_swap::{PancakeSwap, PancakeSwapConfig},
//         TxMonitor, TxMonitorExt,
//     },
// };
//
// use std::{
//     io::Result,
//     path::{Path, PathBuf},
//     sync::Arc,
// };
//
// use anyhow::anyhow;
// use ethers::{
//     core::k256::ecdsa::SigningKey,
//     providers::{Http, JsonRpcClient, Middleware, Provider, PubsubClient, Ws},
//     signers::{Signer, Wallet},
// };
// use futures::{
//     future::{self, try_join},
//     stream::TryStreamExt,
// };
// use metrics::register_counter;
// use serde::Deserialize;
// use tokio::fs;
// use tokio_stream::wrappers::ReadDirStream;
// use tokio_util::sync::CancellationToken;
// use tracing::info;
// use url::Url;

// use crate::account::Account;
//
// use self::abort::{AbortSet, FutureExt as AbortFutureExt};
// // use self::account::Accounts;
// use self::account::Accounts;
// use self::metriced::{
//     FuturesOrdered as MetricedFuturesOrdered, FuturesUnordered as MetricedFuturesUnordered,
// };
// use self::monitors::TxMonitor;
// use self::monitors::{
//     pancake_swap::{PancakeSwap, PancakeSwapConfig},
//     TryMonitor,
//     // Monitor, StatelessBlockMonitor, TxMonitor,
// };
// use self::timed::TryFutureExt as TimedTryFutureExt;
//
// use std::cmp::Reverse;
// use std::path::PathBuf;
// use std::sync::Arc;
//
// use anyhow::{anyhow, Context};
// use ethers::prelude::k256::ecdsa::SigningKey;
// use ethers::prelude::*;
// use futures::channel::mpsc;
// use futures::stream::{self, AbortHandle};
// use futures::{
//     future::{self, join, try_join, AbortRegistration, Aborted, FutureExt, TryFutureExt},
//     pin_mut, select_biased,
//     stream::{FusedStream, FuturesUnordered, Stream, StreamExt, TryStreamExt},
// };
// use futures::{sink, Future, TryFuture};
// use metrics::{register_counter, register_gauge, register_histogram, Counter, Histogram};
// use serde::Deserialize;
// use tokio::fs;
// use tokio_stream::wrappers::ReadDirStream;
// use tokio_util::sync::CancellationToken;
// use tracing::{error, info, trace, warn};
// use url::Url;

// struct ProcessMetrics {
//     seen_txs: Counter,
//     resolved_txs: Counter,
//     resolved_as_pending_txs: Counter,
//     process_tx_duration: Histogram,
//     missed_txs: Counter,
//
//     height: Counter,
//     process_block_duration: Histogram,
// }
//
// impl ProcessMetrics {
//     fn new() -> Self {
//         Self {
//             seen_txs: register_counter!("sandwitch_seen_txs"),
//             resolved_txs: register_counter!("sandwitch_resolved_txs"),
//             resolved_as_pending_txs: register_counter!("sandwitch_resolved_as_pending_txs"),
//             process_tx_duration: register_histogram!("sandwitch_process_tx_duration_seconds"),
//             missed_txs: register_counter!("sandwitch_missed_txs"),
//             height: register_counter!("sandwitch_height"),
//             process_block_duration: register_histogram!("sandwitch_process_block_duration_seconds"),
//         }
//     }
// }

//
// impl<ST, RT, S> App<ST, RT, S>
// where
//     ST: PubsubClient,
//     RT: JsonRpcClient,
//     S: Signer,
// {
//     pub async fn run(&mut self, cancel: &CancellationToken) -> anyhow::Result<()> {
//         let cancelled = cancel.cancelled();
//         pin_mut!(cancelled);
//
//         let (mut new_blocks, mut pending_tx_hashes) = try_join(
//             self.streaming
//                 .subscribe_blocks()
//                 .inspect_ok(|st| info!(subscription_id = %st.id, "subscribed to new blocks"))
//                 .map(|r| r.with_context(|| "failed to subscribe to new blocks")),
//             self.streaming
//                 .subscribe_pending_txs()
//                 .inspect_ok(
//                     |st| info!(subscription_id = %st.id, "subscribed to new pending transactions"),
//                 )
//                 .map(|r| r.with_context(|| "failed to subscribe to new pending transactions")),
//         )
//         .with_abort(cancelled.map(|_| Aborted))
//         .await??;
//
//         let cancelled = cancel.cancelled();
//         pin_mut!(cancelled);
//
//         // let r = self
//         //     .process(
//         //         new_blocks.by_ref(),
//         //         pending_tx_hashes
//         //             .by_ref()
//         //             .take_until(cancelled.inspect(|_| info!("stopping transactions stream..."))),
//         //     )
//         //     .await;
//
//         join(
//             pending_tx_hashes.unsubscribe().map(|r| match r {
//                 Err(err) => error!(%err, "failed to unsubscribe from new pending transactions"),
//                 Ok(_) => info!("unsubscribed from new pending transactions"),
//             }),
//             new_blocks.unsubscribe().map(|r| match r {
//                 Err(err) => error!(%err, "failed to unsubscribe from new blocks"),
//                 Ok(_) => info!("unsubscribed from new blocks"),
//             }),
//         )
//         .await;
//         // r
//     }
//
//     async fn try_process1(
//         txs: impl Stream<Item = TxHash>,
//         blocks: impl Stream<Item = Block<TxHash>>,
//     ) -> anyhow::Result<()> {
//         let (mut txs, htxs) = stream::abortable(txs);
//         let (mut blocks, hblocks) = stream::abortable(blocks);
//
//         let (s, mut r) = mpsc::unbounded();
//         // TODO: drop s
//
//         loop {
//             select_biased! {
//                 r = r.select_next_some() => {
//                     // TODO:
//                 },
//                 block = blocks.select_next_some() => {
//                     // TODO: spawn future resolving the block
//                     // if there was already uncompleted future esolving block,
//                     // then warn about it, wait until completes and spawn new
//                     // TODO: cancel processing of all already mined txs
//                     // this should be done with cancellation token, since
//                     // simple dropping the furure/task would not be ok if monitor
//                     // right now is doing some important work, e.g.: sending a tx
//                 },
//                 tx_hash = txs.select_next_some() => {
//                     // tokio::spawn(self.monitors.)
//                     // TODO: check if there is already processing tx with this hash
//                     // TODO: all monitors should tell current block hash/number when
//                     // processing txs, so that it will use only info from that block
//                 },
//                 complete => {},
//             }
//         }
//     }
//
//     async fn try_process<I, St, F, Fut>(
//         mut stream: St,
//         mut f: F,
//         cancel: CancellationToken,
//     ) -> Result<(), Fut::Error>
//     where
//         St: Stream<Item = I> + Unpin,
//         F: FnMut(I, CancellationToken) -> Option<Fut>,
//         Fut: TryFuture<Ok = ()>,
//         // Fut::Error: Send,
//     {
//         let (mut stream, h) = stream::abortable(stream);
//         let (s, mut r) = mpsc::unbounded();
//
//         let mut first_err = None;
//         loop {
//             select_biased! {
//                 err = r.select_next_some() => {
//                     if first_err.is_some() {
//                         // TODO: log all other errs
//                         continue;
//                     }
//                     first_err = Some(err);
//                     h.abort();
//                     cancel.cancel();
//
//                 },
//                 input = stream.select_next_some() => {
//                     if let Some(t) = f(input, cancel.child_token()) {
//                         tokio::spawn({
//                             let s = s.clone();
//                             async move {
//                                 if let Err(err) = t.into_future().await {
//                                     s.unbounded_send(err).unwrap();
//                                 }
//                             }
//                         });
//                     }
//                 },
//                 complete => return first_err.map_or(Ok(()), Err),
//             }
//         }
//     }
//
//     async fn try_process_txs(mut txs: impl Stream<Item = TxHash>) -> anyhow::Result<()> {
//         // Self::try_process(txs, |tx_hash, cancel| {}, cancel)
//     }
//
//     async fn process_txs(
//         &self,
//         mut pending_txs: impl Stream<Item = TxHash> + Unpin,
//         cancel: CancellationToken,
//     ) -> anyhow::Result<()> {
//         let mut tasks = FuturesUnordered::new();
//
//         loop {
//             select_biased! {
//                 r = tasks.select_next_some() => {
//                     if let Err(e) = r {
//                         let _ = tasks.try_collect::<()>().await;
//                         return Err(e);
//                     }
//                 },
//                 tx = pending_txs.select_next_some() => {
//                     tasks.push(tokio::spawn( // TODO: get tx
//                         self.monitors.process(tx, cancel.child_token()),
//                     ).map(Result::flatten));
//                 }
//                 complete => return Ok(()),
//             }
//         }
//     }
//
//     async fn process(
//         &self,
//         blocks: impl Stream<Item = Block<TxHash>> + Unpin,
//         mut pending_txs: impl Stream<Item = TxHash> + Unpin,
//         cancel: CancellationToken,
//     ) -> anyhow::Result<()> {
//         let process_txs = tokio::spawn(async move {
//             let mut tasks = FuturesUnordered::new();
//
//             loop {
//                 select_biased! {
//                     r = tasks.select_next_some() => {
//                         if let Err(e) = r {
//                             let _ = tasks.try_collect::<()>().await;
//                             return Err(e);
//                         }
//                     },
//                     tx = pending_txs.select_next_some() => {
//                         tasks.push(tokio::spawn(self.monitors.process(tx)).map(Result::flatten));
//                     }
//                     complete => return Ok(()),
//                 }
//             }
//         })
//         .map(Result::flatten);
//
//         // let process_blocks = tokio::spawn(async {
//         //     while let Some(block) = blocks.next().await {
//         //         tokio::spawn(self.monitors.process(block, )); // cancel
//         //     }
//         // });
//         //
//         //
//         // // TODO: tokio unconstrained process blocks
//         // // TODO: filter out txs sent by us
//         // let mut new_blocks = blocks
//         //     .filter(|b| future::ready(b.hash.is_some() && b.number.is_some()))
//         //     .fuse();
//         //
//         // let mut aborts = AbortSet::default();
//         // let mut processing_txs =
//         //     MetricedFuturesUnordered::new(register_gauge!("sandwitch_processing_txs"));
//         // let mut sending_txs =
//         //     MetricedFuturesUnordered::new(register_gauge!("sandwitch_sending_txs"));
//         // let mut processing_blocks =
//         //     MetricedFuturesOrdered::new(register_gauge!("sandwitch_processing_blocks"));
//         //
//         // while !(pending_txs.is_terminated()
//         //     && sending_txs.is_empty()
//         //     && processing_txs.is_empty()
//         //     && processing_blocks.is_empty())
//         // {
//         //     select_biased! {
//         //         ptx = sending_txs.select_next_some() => {
//         //             let ptx: PendingTransaction<_> = ptx?;
//         //             trace!(tx_hash = ?ptx.tx_hash(), "transaction sent");
//         //         },
//         //         to_send = processing_txs.select_next_some() => {
//         //             let to_send: Vec<Bytes> = to_send?;
//         //             // TODO: send in batch
//         //             sending_txs.extend(to_send.into_iter().map(|tx| {
//         //                 self.requesting
//         //                     .send_raw_transaction(tx)
//         //                     .map(|r| r.with_context(|| "failed to send transaction"))
//         //             }));
//         //         },
//         //         r = processing_blocks.select_next_some() => r?,
//         //         block = new_blocks.select_next_some() => {
//         //             // TODO: drop only from this block
//         //             // TODO: restart others
//         //             aborts.abort_all().for_each(drop);
//         //
//         //             let block_hash = block.hash.unwrap();
//         //             let block_number = block.number.unwrap().as_u64();
//         //             self.process_metrics.height.absolute(block_number);
//         //             trace!(?block_hash, ?block_number, "got new block");
//         //             // TODO: metrics: gas avg (prcentil)
//         //
//         //             processing_blocks.push_back(self.process_block(block));
//         //         },
//         //         tx_hash = pending_txs.select_next_some() => {
//         //             let abort = match aborts.try_insert(tx_hash) {
//         //                 Ok(v) => v,
//         //                 Err(tx_hash) => {
//         //                     warn!(
//         //                         ?tx_hash,
//         //                         "this transaction has already been seen, skipping...",
//         //                     );
//         //                     continue;
//         //                 },
//         //             };
//         //             self.process_metrics.seen_txs.increment(1);
//         //             trace!(?tx_hash, "got new transaction");
//         //
//         //             processing_txs.push(self.process_tx(tx_hash, abort));
//         //         },
//         //         complete => break,
//         //     }
//         // }
//         // Ok(())
//     }
//
//     #[tracing::instrument(level = "ERROR", skip_all, fields(
//         block_hash = ?block.hash.unwrap(),
//         block_number = block.number.unwrap().as_u64(),
//     ))]
//     async fn process_block(&self, block: Block<TxHash>) -> anyhow::Result<()> {
//         let block_hash = block.hash.unwrap();
//         let block = match self
//             .requesting
//             .get_block(block_hash)
//             .await
//             .with_context(|| format!("failed to get block by hash: {block_hash:#x}"))?
//         {
//             Some(block) => block,
//             None => {
//                 warn!("fake block, skipping...");
//                 return Ok(());
//             }
//         };
//         let r = {
//             let r = self.monitors.on_block(&block).try_timed().await?;
//             let elapsed = r.elapsed();
//             self.process_metrics.process_block_duration.record(elapsed);
//             trace!(elapsed_ms = elapsed.as_millis(), "block processed");
//             r.into_inner()
//         };
//         Ok(r)
//     }
//
//     #[tracing::instrument(level = "ERROR", skip_all, fields(?tx_hash))]
//     async fn process_tx(
//         &self,
//         tx_hash: TxHash,
//         abort: AbortRegistration,
//     ) -> anyhow::Result<Vec<Bytes>> {
//         match self
//             .get_and_process_tx(tx_hash)
//             .try_timed()
//             .with_abort_reg(abort)
//             .await
//         {
//             Ok(r) => {
//                 let r = r?;
//                 let elapsed = r.elapsed();
//                 self.process_metrics.process_tx_duration.record(elapsed);
//                 trace!(elapsed_ms = elapsed.as_millis(), "transaction processed");
//                 Ok(r.into_inner())
//             }
//             Err(Aborted) => {
//                 self.process_metrics.missed_txs.increment(1);
//                 trace!("transaction has been missed");
//                 Ok([].into())
//             }
//         }
//     }
//
//     async fn get_and_process_tx(&self, tx_hash: TxHash) -> anyhow::Result<Vec<Bytes>> {
//         let tx = match self
//             .requesting
//             .get_transaction(tx_hash)
//             .await
//             .with_context(|| format!("failed to get transaction by hash: {tx_hash:#x}"))?
//         {
//             Some(tx) => {
//                 self.process_metrics.resolved_txs.increment(1);
//
//                 if let Some(block_hash) = tx.block_hash {
//                     trace!(?block_hash, "transaction resolved as already mined");
//                     return Ok([].into());
//                 }
//                 self.process_metrics.resolved_as_pending_txs.increment(1);
//                 trace!("transaction resolved as pending");
//
//                 if tx.value.is_zero()
//                     || tx.gas.is_zero()
//                     || tx.gas_price.map_or(true, |g| g.is_zero())
//                 {
//                     trace!("transaction has been filtered out because of zero value/gas/gas_price");
//                     // TODO: filter out small gas_price
//                     return Ok([].into());
//                 }
//                 tx
//             }
//             None => {
//                 trace!("fake transaction, skipping...");
//                 return Ok([].into());
//             }
//         };
//
//         self.monitors.on_tx(&tx).await
//         // TODO: send txs not in monitor, and abort and restart currently processing txs after each
//         // block
//         // // TODO: make sure that gas_price is some
//         //
//         // // sort desc by gas_price
//         // to_send.sort_by_key(|tx| Reverse(tx.gas_price.unwrap()));
//         //
//         // // TODO: send in batch
//         // // TODO: filter out txs sent by us
//         // let next_nonce = self.next_nonce();
//         // to_send
//         //     .into_iter()
//         //     .enumerate()
//         //     .map(|(i, tx)| tx.nonce(next_nonce + i as u64))
//         //     .map(|tx| self.requesting.send_transaction(tx, None))
//         //     .collect::<FuturesUnordered<_>>()
//         //     .inspect_ok(|ptx| {
//         //         // TODO: log
//         //     })
//         //     .map_ok(|_| ())
//         //     .try_collect::<()>()
//         //     .await
//         //     .map_err(Into::into)
//     }
// }
