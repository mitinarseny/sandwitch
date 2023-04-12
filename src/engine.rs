use core::pin::pin;

use std::{
    cmp::Reverse,
    fmt::format,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use anyhow::{anyhow, Context};
use contracts::multicall::MultiCallContract;
use ethers::{
    providers::{
        Middleware, PendingTransaction, Provider, ProviderError, PubsubClient, SubscriptionStream,
    },
    signers::{LocalWallet, Signer, Wallet},
    types::{
        transaction::eip2718::TypedTransaction, Address, Block, BlockNumber, Bytes, Filter,
        Signature, TxHash, H256, U256,
    },
    utils::keccak256,
};
use futures::{
    future::{self, try_join3, Aborted, Fuse, FusedFuture, FutureExt, TryFuture, TryFutureExt},
    select_biased,
    stream::{FuturesOrdered, FuturesUnordered, StreamExt},
    try_join, Future, Stream,
};
use itertools::Itertools;
use metrics::{register_counter, register_gauge, register_histogram, Counter, Histogram};
use serde::Deserialize;
use serde_with::{serde_as, DurationMilliSeconds};
use tokio::{
    self,
    task::JoinError,
    time::{error::Elapsed, sleep, sleep_until, timeout_at, Duration, Instant},
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, instrument, trace, warn, Instrument, Span};

use crate::{
    abort::FutureExt as AbortFutureExt,
    monitors::{PendingBlock, PendingBlockMonitor, PendingTxMonitor, PrioritizedMultiCall},
    timed::StreamExt as TimedStreamExt,
    //     abort::{AbortSet, FutureExt as AbortFutureExt, MetricedFuturesQueue},
    //     accounts::Accounts,
    //     monitors::{BlockMonitor, BlockMonitorExt, TxMonitor},
    //     timed::TryFutureExt as TryTimedFutureExt,
    providers::timeout::TimeoutProvider,
    transactions::{Transaction, TransactionRequest},
};

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct EngineConfig {
    #[serde(rename = "tx_propagation_delay_ms")]
    #[serde_as(as = "DurationMilliSeconds")]
    pub tx_propagation_delay: Duration,
    pub multicall: Address,
}

pub(crate) struct Engine<P, M> {
    client: Arc<Provider<TimeoutProvider<P>>>,
    tx_propagation_delay: Duration,
    multicall: MultiCallContract<Arc<Provider<TimeoutProvider<P>>>>,
    wallet: LocalWallet,
    next_nonce: AtomicU64,
    // accounts: Arc<Accounts<TimeoutProvider<P>, S>>,
    monitor: M,
}

impl<P, M> Engine<P, M>
where
    P: PubsubClient + 'static,
    P::Error: Send + Sync + 'static,
    M: PendingBlockMonitor,
{
    pub(crate) async fn new(
        client: impl Into<Arc<Provider<TimeoutProvider<P>>>>,
        cfg: EngineConfig,
        wallet: impl Into<LocalWallet>,
        monitor: M,
    ) -> anyhow::Result<Self> {
        let client = client.into();
        let wallet = wallet.into();
        let multicall = MultiCallContract::new(cfg.multicall, client.clone());
        // let multicall_owner = multicall
        //     .owner()
        //     .await
        //     .with_context(|| format!("failed to get owner of {multicall:?}"))?;
        // if wallet.address() != multicall_owner {
        //     return Err(anyhow!(
        //         "{multicall:?} is owned by {multicall_owner:?}, but given wallet has address {:?}",
        //         wallet.address()
        //     ));
        // }
        Ok(Self {
            multicall,
            client,
            tx_propagation_delay: cfg.tx_propagation_delay,
            wallet,
            next_nonce: Default::default(),
            monitor,
        })
    }

    pub fn account(&self) -> Address {
        self.wallet.address()
    }

    pub async fn run(self, cancel: CancellationToken) -> anyhow::Result<()> {
        // TODO: check there is no pending txs yet
        let cancelled = pin!(cancel.cancelled());
        let blocks = self
            .client
            .subscribe_blocks()
            .with_abort(cancelled.map(|_| Aborted))
            .await?
            .with_context(|| "failed to subscribe to new blocks")?;
        debug!(subscription_id = %blocks.id, "subscribed to new blocks");

        let cancelled = pin!(cancel.cancelled());
        let mut blocks = blocks
            .inspect(|block| {
                debug!(
                    block_hash = ?block.hash.unwrap(),
                    block_number = ?block.number.unwrap(),
                    parent_block_hash = ?block.parent_hash,
                    "new head",
                );
            })
            .timed()
            .take_until(cancelled);
        let mut process_pending_block = pin!(Fuse::terminated());
        let mut send_txs = FuturesOrdered::new();

        loop {
            select_biased! {
                sent_tx = send_txs.select_next_some() => {
                    sent_tx?;
                    // TODO: watch sent txs to see if they have been included in processed pending block
                },
                (block, received_at) = blocks.select_next_some() => {
                    if !process_pending_block.is_terminated() {
                        warn!("new block came too early, \
                            aborting processing of previous pending block...");
                        process_pending_block.set(Fuse::terminated());
                        // TODO: check for parent and uncles
                    }

                    if !send_txs.is_empty() {
                        // TODO: warn that we are still sending transactions
                        warn!("we are still sending transactions, skipping this block");
                        continue;
                    }

                    let pending_txs_count = self.get_pending_txs_count_at(&block).await?;
                    if pending_txs_count > 0 {
                        warn!(
                            pending_txs_count,
                            account = ?self.account(),
                            "there are still pending transactions from our account, \
                                waiting for them to be included in one of next blocks...",
                        );
                        // TODO: adjust delays, so we can catch on next time
                        continue;
                    }
                    process_pending_block.set(
                        self.process_pending_block(block, received_at)
                            .fuse(),
                    );
                },
                processed_block = &mut process_pending_block => {
                    let pending_block = processed_block?;
                    debug!("pending block processed");
                    send_txs.extend(
                        self.extract_txs_to_send(pending_block)
                            .map(|tx| self.sign_and_send(tx)),
                    );
                },
                complete => return Ok(()),
            }
        }
    }

    async fn get_pending_txs_count_at(&self, new_block: &Block<TxHash>) -> anyhow::Result<u64> {
        let (next, pending) = try_join!(
            self.client
                // .get_transaction_count(self.account(), Some(new_block.hash.unwrap().into())),
                .get_transaction_count(self.account(), Some(BlockNumber::Latest.into())),
            self.client
                .get_transaction_count(self.account(), Some(BlockNumber::Pending.into())),
        )?;
        if pending == next {
            self.next_nonce.store(next.as_u64(), Ordering::SeqCst);
        }
        Ok((pending - next).as_u64())
    }

    async fn get_pending_block(&self) -> Result<PendingBlock, ProviderError> {
        let log_filter = Filter::new().select(BlockNumber::Pending);
        let (block, logs) = try_join!(
            self.client.get_block_with_txs(BlockNumber::Pending),
            future::ok(Vec::new()), // TODO
                                    // self.client.get_logs(&log_filter),
        )?;
        let Some(block) = block else {
            warn!("pending block doest not exist");
            return Err(ProviderError::UnsupportedRPC);
        };
        // TODO: check that it is parent of last mined block
        Ok(PendingBlock::new(block, logs, 0)) // TODO: priority_fee
    }

    #[instrument(err, skip_all, fields(parent_hash = ?latest_block.hash.unwrap()))]
    async fn process_pending_block(
        &self,
        latest_block: Block<TxHash>,
        received_at: Instant,
    ) -> anyhow::Result<PendingBlock> {
        let next_block_at = self.estimate_next_block_at(received_at);
        // TODO: half time
        sleep_until(next_block_at - Duration::from_secs(1)).await;
        // TODO: get account balance and check if there is enoght ETH to send txs
        let block = self.get_pending_block().await?;
        match timeout_at(
            next_block_at - self.tx_propagation_delay, // TODO: subtract latency
            self.monitor.process_pending_block(&block),
        )
        .await
        {
            Ok(r) => r?,
            Err(_) => warn!("elapsed"), // TODO
        }
        Ok(block)
    }

    fn estimate_next_block_at(&self, received_at: Instant) -> Instant {
        // TODO
        received_at + Duration::from_secs(3)
    }

    fn extract_txs_to_send(&self, block: PendingBlock) -> impl Iterator<Item = TransactionRequest> {
        // TODO: auto add transfer to the account to pay for gas
        block
            .into_send_txs()
            .into_iter()
            .group_by(|t| t.priority_fee)
            .into_iter()
            .map(|(priority_fee, group)| PrioritizedMultiCall {
                calls: group.map(|g| g.calls).concat(),
                priority_fee,
            })
            .sorted_unstable_by_key(|tx| Reverse(tx.priority_fee))
            .map(|p| TransactionRequest::default()) // TODO
    }

    #[instrument(err, skip_all, fields(
        from = ?tx.from.unwrap(),
        gas = ?tx.gas.unwrap(),
        tx_hash, nonce
    ))]
    async fn sign_and_send(&self, tx: TransactionRequest) -> anyhow::Result<Transaction> {
        // TODO: set and increase nonce
        let tx = tx.into();
        let signature = self.wallet.sign_transaction_sync(&tx)?;
        let tx = match tx {
            #[cfg(not(feature = "legacy"))]
            TypedTransaction::Eip1559(tx) => tx,
            #[cfg(feature = "legacy")]
            TypedTransaction::Legacy(tx) => tx,
            _ => unreachable!(),
        };
        let raw = tx.rlp_signed(&signature);
        let tx = Transaction::from_request(tx, signature)?;
        Span::current()
            .record("tx_hash", format!("{:?}", tx.hash))
            .record("nonce", format!("{}", tx.nonce));
        let pending_tx = self.client.send_raw_transaction(raw).await?;
        Ok(tx)
    }

    // async fn sign_and_send(&self, txs: impl IntoIterator<Item = TransactionRequest>) ->

    // pub(crate) async fn run1(&mut self, cancel: CancellationToken) -> anyhow::Result<()> {
    //     let cancelled = cancel.cancelled().fuse();
    //     pin_mut!(cancelled);

    //     let blocks = self
    //         .client
    //         .subscribe_blocks()
    //         .inspect_ok(|st| debug!(subscription_id = %st.id, "subscribed to new blocks"))
    //         .map(|r| r.with_context(|| "failed to subscribe to new blocks"))
    //         .await?; // TODO: with_abort

    //     let blocks = blocks
    //         .fuse()
    //         .take_while(|_| future::ready(!cancel.is_cancelled()));

    //     let request_pendings = sleep(Duration::from_secs(9999));
    //     pin_mut!(request_pendings);

    //     while !(blocks.is_terminated()) {
    //         select_biased! {
    //             block = blocks.select_next_some() => {
    //                 // TODO: request account balance every time new block arrives
    //                 // to ensure that we have enough gas to send txs
    //                 request_pendings.as_mut().reset(Instant::now() + Duration::from_secs(3));
    //             },
    //             () = &mut request_pendings => {

    //             },
    //             complete => break,
    //         }
    //     }

    //     Ok(())

    //     // TODO: subscribe logs about successful front-run events
    //     // get logs topics from monitors

    //     // TODO: real-time calculate latency

    //     // let (blocks, last_block) = try_join3(
    //     //     self.client
    //     //         .subscribe_blocks()
    //     //         .inspect_ok(|st| info!(subscription_id = %st.id, "subscribed to new blocks"))
    //     //         .map(|r| r.with_context(|| "failed to subscribe to new blocks")),
    //     //     self.client
    //     //         .subscribe_pending_txs()
    //     //         .inspect_ok(
    //     //             |st| info!(subscription_id = %st.id, "subscribed to new pending transactions"),
    //     //         )
    //     //         .map(|r| r.with_context(|| "failed to subscribe to new pending transactions")),
    //     //     self.client
    //     //         .get_block_with_txs(BlockNumber::Latest)
    //     //         .map_ok(Option::unwrap)
    //     //         .err_into::<anyhow::Error>(),
    //     // )
    //     // .with_abort(cancelled.map(|_| Aborted))
    //     // .await??;
    //     //
    //     // let mut last_block_hash = last_block.hash.unwrap();
    //     // info!(?last_block_hash, "starting from last block...");
    //     // self.metrics.new_block(&last_block);
    //     // self.metrics.block_valid(&last_block);
    //     //
    //     // let tx_hashes = tx_hashes
    //     //     .fuse()
    //     //     .take_while(|_| future::ready(!cancel.is_cancelled()));
    //     // let mut txs = AbortSet::new(
    //     //     MetricedFuturesQueue::<FuturesUnordered<_>, _>::new_with_default(register_gauge!(
    //     //         "sandwitch_resolving_txs"
    //     //     )),
    //     // );
    //     // let mut process_txs = AbortSet::new(
    //     //     MetricedFuturesQueue::<FuturesUnordered<_>, _>::new_with_default(register_gauge!(
    //     //         "sandwitch_processing_txs"
    //     //     )),
    //     // );
    //     //
    //     // let blocks = blocks
    //     //     .fuse()
    //     //     .take_while(|_| future::ready(!cancel.is_cancelled()));
    //     // let mut blocks_with_txs = AbortSet::new(
    //     //     MetricedFuturesQueue::<FuturesOrdered<_>, _>::new_with_default(register_gauge!(
    //     //         "sandwitch_resolving_blocks"
    //     //     )),
    //     // );
    //     // let mut process_blocks = AbortSet::new(
    //     //     MetricedFuturesQueue::<FuturesOrdered<_>, _>::new_with_default(register_gauge!(
    //     //         "sandwtich_processing_blocks"
    //     //     )),
    //     // );
    //     //
    //     // let cancelled = cancel.cancelled().fuse();
    //     // pin_mut!(tx_hashes, blocks, cancelled);
    //     //
    //     // let mut first_err: Option<anyhow::Error> = None;
    //     // let mut fatal_err = |err| {
    //     //     if first_err.is_none() {
    //     //         cancel.cancel();
    //     //         first_err = Some(err);
    //     //     }
    //     // };
    //     //
    //     // while !(tx_hashes.is_terminated()
    //     //     && blocks.is_terminated()
    //     //     && process_txs.is_terminated()
    //     //     && process_blocks.is_terminated())
    //     // {
    //     //     select_biased! {
    //     //         _ = cancelled => {
    //     //             txs.abort_all();
    //     //             process_txs.abort_all();
    //     //             blocks_with_txs.abort_all();
    //     //             process_blocks.abort_all();
    //     //         },
    //     //         (r, block_hash) = process_blocks.select_next_some() => {
    //     //             let r: Result<anyhow::Result<(_, Duration)>, JoinError> = r;
    //     //             match r.map_err(Into::into).flatten() {
    //     //                 Ok((_, elapsed)) => {
    //     //                     trace!(?block_hash, ?elapsed, "block proceesed");
    //     //                     self.metrics.block_processed(elapsed);
    //     //                 },
    //     //                 Err(err) => {
    //     //                     error!(?block_hash, "block processing failed: {err:#}");
    //     //                     fatal_err(err.context(format!(
    //     //                         "block processing failed for: {block_hash:?}")));
    //     //                 },
    //     //             }
    //     //         },
    //     //         (r, tx_hash) = process_txs.select_next_some() => {
    //     //             let r: Result<anyhow::Result<(_, Duration)>, JoinError> = r;
    //     //             match r.map_err(Into::into).flatten() {
    //     //                 Ok((_, elapsed)) => {
    //     //                     trace!(?tx_hash, ?elapsed, "transaction processed");
    //     //                     self.metrics.tx_processed(elapsed);
    //     //                 },
    //     //                 Err(err) => {
    //     //                     error!(?tx_hash, "transaction processing failed: {err:#}");
    //     //                     fatal_err(err.context(format!(
    //     //                         "transaction processing failed for: {tx_hash:?}")));
    //     //                 },
    //     //             }
    //     //         },
    //     //         (block, block_hash) = blocks_with_txs.select_next_some() => {
    //     //             let block: Result<(Option<Block<Transaction>>, Duration), _> = block;
    //     //             match block {
    //     //                 Ok((block, elapsed)) => {
    //     //                     trace!(?block_hash, ?elapsed, "block resolved");
    //     //                     self.metrics.block_resolved(elapsed);
    //     //                     let Some(block): Option<Block<Transaction>> = block else {
    //     //                         warn!(?block_hash, "invalid block, skipping...");
    //     //                         continue;
    //     //                     };
    //     //                     let Ok(h) = process_blocks.try_insert(block_hash) else {
    //     //                         warn!(
    //     //                             ?block_hash,
    //     //                             "this block is already being processed, skipping...",
    //     //                         );
    //     //                         continue;
    //     //                     };
    //     //                     // TODO: check block timestamp?
    //     //
    //     //                     last_block_hash = block_hash;
    //     //                     self.metrics.block_valid(&block);
    //     //                     for tx in &block.transactions {
    //     //                         if txs.abort(&tx.hash)
    //     //                             .or(process_txs.abort(&tx.hash))
    //     //                             .is_some() {
    //     //                             debug!(
    //     //                                 ?tx.hash,
    //     //                                 ?block_hash,
    //     //                                 "transaction has just been mined, \
    //     //                                     cancelling its processing...",
    //     //                             );
    //     //                             self.metrics.tx_missed();
    //     //                         };
    //     //                     }
    //     //                     h.spawn(self.process_block(block).into_future().try_timed());
    //     //                 },
    //     //                 Err(err) => {
    //     //                     error!(?block_hash, "failed to resolve block: {err:#}");
    //     //                     if let ProviderError::JsonRpcClientError(err) = &err {
    //     //                         if let Some(TimeoutProviderError::<P>::Timeout(_)) =
    //     //                             err.downcast_ref() {
    //     //                             continue;
    //     //                         }
    //     //                     }
    //     //                     fatal_err(anyhow::Error::from(err)
    //     //                         .context(format!("failed to resolve block: {block_hash:?}")));
    //     //                 },
    //     //             }
    //     //         },
    //     //         block = blocks.select_next_some() => {
    //     //             let block_hash = block.hash.unwrap();
    //     //             if process_blocks.contains(&block_hash) {
    //     //                 warn!(
    //     //                     ?block_hash,
    //     //                     "received block is already being processing, skipping...",
    //     //                 );
    //     //                 continue;
    //     //             }
    //     //             let Ok(h) = blocks_with_txs.try_insert(block_hash) else {
    //     //                 warn!(
    //     //                     ?block_hash,
    //     //                     "received block is already being resolved, skipping...",
    //     //                 );
    //     //                 continue;
    //     //             };
    //     //             trace!(
    //     //                 ?block_hash,
    //     //                 block_number = block.number.unwrap().as_u64(),
    //     //                 "received new block"
    //     //             );
    //     //             self.metrics.new_block(&block);
    //     //             h.insert(self.client.get_block_with_txs(block_hash).try_timed());
    //     //         },
    //     //         (tx, tx_hash) = txs.select_next_some() => {
    //     //             let tx: Result<(Option<Transaction>, Duration), _> = tx;
    //     //             match tx {
    //     //                 Ok((tx, elapsed)) => {
    //     //                     self.metrics.tx_resolved(elapsed);
    //     //                     trace!(?tx_hash, ?elapsed, "transaction resolved");
    //     //                     let Some(tx) = tx else {
    //     //                         trace!(?tx_hash, "invalid tx, skipping...");
    //     //                         continue;
    //     //                     };
    //     //                     self.metrics.tx_valid();
    //     //                     if let Some(block_hash) = tx.block_hash {
    //     //                         trace!(
    //     //                             ?tx_hash,
    //     //                             ?block_hash,
    //     //                             "transaction resolved as already mined, skipping...",
    //     //                         );
    //     //                         continue;
    //     //                     }
    //     //                     let Ok(h) = process_txs.try_insert(tx_hash) else {
    //     //                         warn!(
    //     //                             ?tx_hash,
    //     //                             "this transaction is already being processed, skipping...",
    //     //                         );
    //     //                         continue;
    //     //                     };
    //     //
    //     //                     trace!(
    //     //                         ?tx_hash,
    //     //                         ?last_block_hash,
    //     //                         "transaction resolved as pending, processing...",
    //     //                     );
    //     //                     self.metrics.tx_resolved_as_pending();
    //     //                     h.spawn(
    //     //                         self.process_tx(tx, last_block_hash)
    //     //                             .into_future()
    //     //                             .try_timed(),
    //     //                     );
    //     //                 },
    //     //                 Err(err) => {
    //     //                     error!(?tx_hash, "failed to resolve transaction: {err:#}");
    //     //                     if let ProviderError::JsonRpcClientError(err) = &err {
    //     //                         if let Some(TimeoutProviderError::<P>::Timeout(_)) =
    //     //                             err.downcast_ref() {
    //     //                             self.metrics.tx_missed();
    //     //                             continue;
    //     //                         }
    //     //                     }
    //     //                     fatal_err(anyhow::Error::from(err).context(format!(
    //     //                         "failed to resolve transaction: {tx_hash:?}")));
    //     //                 },
    //     //             }
    //     //         },
    //     //         tx_hash = tx_hashes.select_next_some() => {
    //     //             if process_txs.contains(&tx_hash) {
    //     //                 warn!(
    //     //                     ?tx_hash,
    //     //                     "received transaction is already being processed, skipping...",
    //     //                 );
    //     //                 continue;
    //     //             }
    //     //
    //     //             let Ok(h) = txs.try_insert(tx_hash) else {
    //     //                 warn!(
    //     //                     ?tx_hash,
    //     //                     "received transaction is already being resolving, skipping...",
    //     //                 );
    //     //                 continue;
    //     //             };
    //     //             trace!(?tx_hash, "received new transaction hash");
    //     //             self.metrics.new_tx();
    //     //
    //     //             h.insert(self.client.get_transaction(tx_hash).try_timed());
    //     //         },
    //     //         complete => break,
    //     //     }
    //     // }
    //     //
    //     // first_err.map_or(Ok(()), Err)
    // }

    // #[tracing::instrument(
    //     skip_all,
    //     fields(
    //         block_hash = ?block.hash.unwrap(),
    //         block_number = block.number.unwrap().as_u64(),
    //     ),
    // )]
    // fn process_block(
    //     &self,
    //     block: Block<Transaction>,
    // ) -> impl TryFuture<Ok = BM::Ok, Error = BM::Error> {
    //     let block_monitor = self.monitor.clone();
    //     let accounts = self.accounts.clone();
    //     async move {
    //         (accounts.map_err(|_| unreachable!()), block_monitor)
    //             .map(|_| ())
    //             .process_block(&block)
    //             .await
    //     }
    //     .in_current_span()
    // }

    // #[tracing::instrument(skip_all, fields(?tx.hash, ?block_hash))]
    // fn process_tx(
    //     &self,
    //     tx: Transaction,
    //     block_hash: H256,
    // ) -> impl TryFuture<Ok = TM::Ok, Error = TM::Error> {
    //     let tx_monitor = self.tx_monitor.clone();
    //     async move { tx_monitor.process_tx(&tx, block_hash).await }.in_current_span()
    // }
}

// struct Metrics {
//     seen_txs: Counter,
//     resolve_tx_duration: Histogram,
//     valid_txs: Counter,
//     resolved_as_pendning_txs: Counter,
//     process_tx_duration: Histogram,
//     missed_txs: Counter,

//     height: Counter,
//     resolve_block_duration: Histogram,
//     block_gas_used: Histogram,
//     block_gas_limit: Histogram,
//     txs_in_block: Histogram,
//     tx_gas_price: Histogram,
//     process_block_duration: Histogram,
// }

// impl Default for Metrics {
//     fn default() -> Self {
//         Self {
//             seen_txs: register_counter!("sandwitch_seen_txs"),
//             resolve_tx_duration: register_histogram!("sandwitch_resolve_tx_duration"),
//             valid_txs: register_counter!("sandwitch_valid_txs"),
//             resolved_as_pendning_txs: register_counter!("sandwitch_resolved_as_pending_txs"),
//             process_tx_duration: register_histogram!("sandwitch_process_tx_duration"),
//             missed_txs: register_counter!("sandwitch_missed_txs"),
//             height: register_counter!("sandwitch_height"),
//             resolve_block_duration: register_histogram!("sandwitch_resolve_block_duration"),
//             block_gas_used: register_histogram!("sandwitch_block_gas_used"),
//             block_gas_limit: register_histogram!("sandwitch_block_gas_limit"),
//             txs_in_block: register_histogram!("sandwitch_txs_in_block"),
//             tx_gas_price: register_histogram!("sandwitch_tx_gas_price"),
//             process_block_duration: register_histogram!("sandwitch_process_block_duration"),
//         }
//     }
// }

// impl Metrics {
//     fn new_tx(&self) {
//         self.seen_txs.increment(1);
//     }

//     fn tx_resolved(&self, elapsed: Duration) {
//         self.resolve_tx_duration.record(elapsed)
//     }

//     fn tx_valid(&self) {
//         self.valid_txs.increment(1);
//     }

//     fn tx_resolved_as_pending(&self) {
//         self.resolved_as_pendning_txs.increment(1);
//     }

//     fn tx_processed(&self, elapsed: Duration) {
//         self.process_tx_duration.record(elapsed);
//     }

//     fn tx_missed(&self) {
//         self.missed_txs.increment(1);
//     }

//     fn new_block<TX>(&self, block: &Block<TX>) {
//         self.height.absolute(block.number.unwrap().as_u64());
//     }

//     fn block_resolved(&self, elapsed: Duration) {
//         self.resolve_block_duration.record(elapsed);
//     }

//     fn block_valid(&self, block: &Block<Transaction>) {
//         self.block_gas_used.record(block.gas_used.as_u128() as f64);
//         self.block_gas_limit
//             .record(block.gas_limit.as_u128() as f64);
//         self.txs_in_block.record(block.transactions.len() as f64);
//         for gas_price in block.transactions.iter().filter_map(|tx| tx.gas_price) {
//             self.tx_gas_price.record(gas_price.as_u128() as f64);
//         }
//     }

//     fn block_processed(&self, elapsed: Duration) {
//         self.process_block_duration.record(elapsed);
//     }
// }
