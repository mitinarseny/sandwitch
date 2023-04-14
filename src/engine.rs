use core::pin::pin;

use std::{
    cmp::Reverse,
    fmt::format,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
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
    future::{Aborted, Fuse, FusedFuture, FutureExt, TryFuture, TryFutureExt},
    select_biased,
    stream::{self, FuturesUnordered, StreamExt},
    try_join,
};
use itertools::Itertools;
// use metrics::{register_counter, register_gauge, register_histogram, Counter, Histogram};
use serde::Deserialize;
use serde_with::{serde_as, DurationMilliSeconds};
use tokio::{
    self,
    time::{sleep_until, timeout_at, Duration, Instant},
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, error_span, field, info, instrument, warn, Span};

use crate::{
    abort::FutureExt as AbortFutureExt,
    monitors::{PendingBlock, PendingBlockMonitor, PrioritizedMultiCall},
    providers::{LatencyProvider, TimeoutProvider},
    timed::StreamExt as TimedStreamExt,
    transactions::{Transaction, TransactionRequest},
};

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct EngineConfig {
    #[serde(rename = "block_interval_ms")]
    #[serde_as(as = "DurationMilliSeconds")]
    pub block_interval: Duration,

    #[serde(rename = "tx_propagation_delay_ms")]
    #[serde_as(as = "DurationMilliSeconds")]
    pub tx_propagation_delay: Duration,

    pub multicall: Address,
}

pub type ProviderStack<P> = Provider<LatencyProvider<TimeoutProvider<P>>>;

pub(crate) struct Engine<P, M> {
    client: Arc<ProviderStack<P>>,
    multicall: MultiCallContract<Arc<ProviderStack<P>>>,
    wallet: LocalWallet,
    next_nonce: AtomicU64,
    tx_propagation_delay: Duration,
    block_interval: Duration,
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
        client: impl Into<Arc<ProviderStack<P>>>,
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
            wallet,
            next_nonce: Default::default(),
            tx_propagation_delay: cfg.tx_propagation_delay,
            block_interval: cfg.block_interval,
            monitor,
        })
    }

    pub fn account(&self) -> Address {
        self.wallet.address()
    }

    pub async fn run(self, cancel: CancellationToken) -> anyhow::Result<()> {
        let mut cancelled = pin!(cancel.cancelled().map(|_| Aborted));
        let blocks = self
            .client
            .subscribe_blocks()
            .with_abort(&mut cancelled)
            .await?
            .with_context(|| "failed to subscribe to new blocks")?;
        debug!(subscription_id = %blocks.id, "subscribed to new blocks");

        let (blocks, blocks_handle) = stream::abortable(blocks.timed());
        let mut blocks = blocks.fuse();
        let mut process_pending_block = pin!(Fuse::terminated());
        // process_pending_block.
        let mut send_txs = FuturesUnordered::new();
        let mut cancelled = pin!(cancel.cancelled().map(|_| Aborted));

        loop {
            select_biased! {
                sent_tx = send_txs.select_next_some() => {
                    sent_tx?;
                    // TODO: watch sent txs to see if they have been included in processed pending block
                },
                _ = &mut cancelled => error_span!("cancelled").in_scope(||{
                    info!("stop receiving new heads...");
                    blocks_handle.abort();
                    if !process_pending_block.is_terminated() {
                        warn!("pending block was being processed, aborting...");
                        process_pending_block.set(Fuse::terminated());
                    }
                    if !send_txs.is_empty() {
                        info!(
                            left_to_send = send_txs.len(),
                            "still sending transactions, waiting for them to finish...",
                        );
                    }
                }),
                (block, received_at) = blocks.select_next_some() => Self::new_head_span(&block).in_scope(|| {
                    debug!("new head received");
                    // TODO: check if block_time is not too small
                    // TODO: check for parent and uncles

                    if !process_pending_block.is_terminated() {
                        warn!("new head came too early, \
                            aborting previous pending block processing...");
                        process_pending_block.set(Fuse::terminated());
                    }
                    if !send_txs.is_empty() {
                        // TODO: warn that we are still sending transactions
                        warn!(
                            left_to_send = send_txs.len(),
                            "we are still sending transactions, skipping this block...",
                        );
                        return;
                    }
                    process_pending_block.set(
                        self.process_pending_block(block, received_at, Span::current())
                            .fuse(),
                    );
                }),
                to_send = &mut process_pending_block => send_txs.extend(
                    to_send?
                        .into_iter()
                        .flatten()
                        .map(TryFutureExt::into_future),
                ),
                complete => return Ok(()),
            }
        }
    }

    fn new_head_span(block: &Block<TxHash>) -> Span {
        error_span!(
            "new head",
            block_hash = ?block.hash.unwrap(),
            block_number = ?block.number.unwrap().as_u64(),
            parent_block_hash = ?block.parent_hash,
        )
    }

    async fn get_pending_txs_count_at(&self, new_block: &Block<TxHash>) -> anyhow::Result<u64> {
        let (next, pending) = try_join!(
            self.client
                .get_transaction_count(self.account(), Some(new_block.hash.unwrap().into())),
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
            self.client.get_logs(&log_filter),
        )?;
        let Some(block) = block else {
            error!("pending block doest not exist");
            return Err(ProviderError::UnsupportedRPC);
        };
        // TODO: check that it is parent of last mined block
        Ok(PendingBlock::new(block, logs, 0)) // TODO: priority_fee
    }

    fn estimate_next_block_at(&self, received_at: Instant) -> Instant {
        received_at + self.block_interval
    }

    fn latency(&self) -> Duration {
        self.client.as_ref().as_ref().latency()
    }

    #[instrument(
        follows_from = [new_head_span],
        skip_all,
        fields(
            parent_block_hash,
            block_number,
        ),
        err,
    )]
    async fn process_pending_block(
        &self,
        latest_block: Block<TxHash>,
        received_at: Instant,
        new_head_span: impl Into<Option<tracing::Id>>,
    ) -> anyhow::Result<
        Option<impl Iterator<Item = impl TryFuture<Ok = Transaction, Error = anyhow::Error> + '_>>,
    > {
        let span = Span::current();

        let pending_txs_count = self.get_pending_txs_count_at(&latest_block).await?;
        if pending_txs_count > 0 {
            warn!(
                pending_txs_count,
                account = ?self.account(),
                "there are still pending transactions from our account, \
                    waiting for them to be included in one of next blocks...",
            );
            // TODO: adjust delays, so we can catch on next time
            return Ok(None);
        }

        let next_block_at = self.estimate_next_block_at(received_at);
        let latency = self.latency();
        let abort_processing_at = next_block_at - self.tx_propagation_delay - latency;

        // TODO: keep track of monitor latency
        sleep_until(abort_processing_at - Duration::from_secs(1)).await;
        debug!("requesting pending block...");
        // TODO: get account balance and check if there is enoght ETH to send txs
        let pending_block = self.get_pending_block().await?;
        span.record("parent_block_hash", field::debug(pending_block.parent_hash))
            .record("block_number", pending_block.number.unwrap().as_u64());
        if !latest_block
            .hash
            .is_some_and(|h| h == pending_block.parent_hash)
        {
            warn!("pending block is not a child of latest, skipping...");
            return Ok(None);
        }
        debug!("processing pending block");
        timeout_at(
            abort_processing_at,
            self.monitor.process_pending_block(&pending_block),
        )
        .unwrap_or_else(|_| {
            warn!("elapsed");
            Ok(())
        })
        .await?;
        debug!("pending block processed");
        // TODO: maybe force sleep until abort_processing_at, so we would send just at the end of the block?
        Ok(Some(
            self.extract_txs_to_send(pending_block)
                .map(move |tx| self.sign_and_send(tx, span.clone())),
        ))
    }

    fn extract_txs_to_send(
        &self,
        block: PendingBlock,
    ) -> impl ExactSizeIterator<Item = TransactionRequest> {
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

    // TODO: instrument gas price / max_fee_*
    #[instrument(
        follows_from = [process_pending_block_span],
        skip_all,
        fields(
            from = ?tx.from.unwrap(),
            gas = ?tx.gas.unwrap(),
            tx_hash, nonce
        ),
        err,
    )]
    async fn sign_and_send(
        &self,
        tx: TransactionRequest,
        process_pending_block_span: impl Into<Option<tracing::Id>>,
    ) -> anyhow::Result<Transaction> {
        // TODO: set and increase nonce
        let tx = tx.into();
        // TODO: debug!("assigned nonce: {nonce}");
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
            .record("tx_hash", field::debug(tx.hash))
            .record("nonce", field::display(tx.nonce));
        let pending_tx = self.client.send_raw_transaction(raw).await?;
        info!("transaction sent");
        Ok(tx)
    }
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
