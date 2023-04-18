use core::pin::pin;
use std::sync::Arc;

use anyhow::{anyhow, Context};
use contracts::{
    multicall::{MultiCall, MultiCallContract},
    EthTypedCall,
};
use ethers::{
    providers::{Middleware, Provider, ProviderError, PubsubClient},
    signers::{LocalWallet, Signer},
    types::{
        transaction::eip2718::TypedTransaction, Address, Block, BlockNumber, Filter, TxHash, U256,
    },
    utils::keccak256,
};
use futures::{
    future::{Aborted, Fuse, FusedFuture, FutureExt, TryFutureExt},
    select_biased,
    stream::{self, FuturesOrdered, FuturesUnordered, StreamExt, TryStreamExt},
    try_join,
};
// use metrics::{register_counter, register_gauge, register_histogram, Counter, Histogram};
use serde::Deserialize;
use serde_with::{serde_as, DurationMilliSeconds};
use tokio::{
    self,
    time::{sleep_until, timeout_at, Duration, Instant},
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, error_span, field, info, instrument, warn, Instrument, Span};

use crate::{
    abort::FutureExt as AbortFutureExt,
    monitors::{PendingBlock, PendingBlockMonitor, PrioritizedMultiCall},
    providers::{LatencyProvider, TimeoutProvider},
    timed::StreamExt as TimedStreamExt,
    transactions::TransactionRequest,
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

// TODO: use Signer Middleware
pub type ProviderStack<P> = Provider<LatencyProvider<TimeoutProvider<P>>>;

pub(crate) struct Engine<P, M> {
    client: Arc<ProviderStack<P>>,
    multicall: MultiCallContract<Arc<ProviderStack<P>>, ProviderStack<P>>,
    wallet: LocalWallet,
    tx_propagation_delay: Duration,
    block_interval: Duration,
    // accounts: Arc<Accounts<TimeoutProvider<P>, S>>,
    monitor: M,
}

impl<P, M> Engine<P, M>
where
    P: PubsubClient + 'static,
    P::Error: Send + Sync + 'static,
    M: PendingBlockMonitor<ProviderStack<P>>,
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
        if !client.mining().await? {
            return Err(anyhow!("node is not mining"));
        }
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
                sent_tx_hash = send_txs.select_next_some() => {
                    sent_tx_hash?;
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
                (block, received_at) = blocks.select_next_some() => {
                    Self::new_head_span(&block).in_scope(|| {
                        debug!("new head received");

                        if !process_pending_block.is_terminated() {
                            warn!("new head came too early, \
                                aborting previous pending block processing...");
                            process_pending_block.set(Fuse::terminated());
                        }
                        if !send_txs.is_empty() {
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
                    });
                },
                to_send = &mut process_pending_block => if let Some((to_send, process_block_span)) = to_send? {
                    send_txs.extend(
                        to_send
                            .into_iter()
                            .map(|tx| self.sign_and_send(tx, process_block_span.clone())),
                    )
                },
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

    async fn get_next_nonce_at(&self, block: &Block<TxHash>) -> anyhow::Result<Option<U256>> {
        let (nonce_at_block, pending_nonce) = try_join!(
            self.client
                .get_transaction_count(self.account(), Some(block.number.unwrap().into())),
            self.client
                .get_transaction_count(self.account(), Some(BlockNumber::Pending.into())),
        )?;
        if pending_nonce < nonce_at_block {
            return Err(anyhow!(
                "pending txs count appears to be less than at some block number"
            ));
        }
        if pending_nonce > nonce_at_block {
            warn!(
                pending_txs_count = (pending_nonce - nonce_at_block).as_u64(),
                account = ?self.account(),
                "there are still pending transactions from our account, \
                    waiting for them to be included in one of next blocks...",
            );
            // TODO: adjust delays, so we can catch on next time
            return Ok(None);
        }
        Ok(Some(nonce_at_block))
    }

    async fn get_pending_block(&self) -> anyhow::Result<PendingBlock<'_, ProviderStack<P>>> {
        let log_filter = Filter::new().select(BlockNumber::Pending);
        let (block, logs) = try_join!(
            self.client.get_block_with_txs(BlockNumber::Pending),
            self.client.get_logs(&log_filter),
        )?;
        let Some(block) = block else {
            error!("pending block doest not exist");
            return Err(ProviderError::UnsupportedRPC.into());
        };
        // TODO: check if not legacy that base_fee_per_gas is not none
        Ok(PendingBlock::try_from(
            block,
            logs,
            None,
            self.account(),
            &self.multicall,
        )?) // TODO: priority_fee
    }

    fn estimate_next_block_at(&self, received_at: Instant) -> Instant {
        received_at + self.block_interval
    }

    async fn latency(&self) -> Duration {
        self.client.as_ref().as_ref().latency().await
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
    ) -> anyhow::Result<Option<(Vec<TypedTransaction>, Span)>> {
        let span = Span::current();

        let Some(next_nonce) = self.get_next_nonce_at(&latest_block).await? else {
            return Ok(None);
        };

        let next_block_at = self.estimate_next_block_at(received_at);
        let latency = self.latency().await;
        let abort_processing_at = next_block_at - self.tx_propagation_delay - 2 * latency;

        // TODO: keep track of monitor latency
        sleep_until(abort_processing_at - Duration::from_secs(1)).await;
        debug!("requesting pending block...");
        // TODO: get account balance and check if there is enoght ETH to send txs
        let pending_block = self.get_pending_block().in_current_span().await?;
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
        // TODO: log to_send count

        Ok(Some((
            self.extract_txs_to_send(pending_block, next_nonce).await?,
            Span::current(),
        )))
    }

    async fn extract_txs_to_send(
        &self,
        processed_block: PendingBlock<'_, ProviderStack<P>>,
        mut next_nonce: U256,
    ) -> anyhow::Result<Vec<TypedTransaction>> {
        processed_block
            .to_send
            .join_adjacent()
            .into_iter()
            .map(|p| {
                self.make_tx(p, &processed_block.block, {
                    let nonce = next_nonce;
                    next_nonce += 1.into();
                    nonce
                })
            })
            .collect::<FuturesOrdered<_>>()
            .try_collect()
            .await
    }

    async fn make_tx<TX>(
        &self,
        p: PrioritizedMultiCall,
        block: &Block<TX>,
        nonce: impl Into<U256>,
    ) -> anyhow::Result<TypedTransaction> {
        // TODO: value?
        let mut tx = TransactionRequest::default()
            .from(self.account())
            .to(self.multicall.address())
            .data({
                let (raw, _meta) = p.calls.into_inner().encode_raw_calls();
                raw.encode_calldata()
            })
            .nonce(nonce)
            .chain_id(self.wallet.chain_id());

        #[cfg(not(feature = "legacy"))]
        {
            tx = tx
                .max_priority_fee_per_gas(p.priority_fee_per_gas)
                .max_fee_per_gas(block.base_fee_per_gas.unwrap() + p.priority_fee_per_gas);
        }
        #[cfg(feature = "legacy")]
        {
            tx = tx.gas_price(p.priority_fee);
        }

        let mut tx: TypedTransaction = tx.into();
        tx.set_gas(
            self.client
                .estimate_gas(&tx, Some(BlockNumber::Pending.into()))
                .await?,
        );
        Ok(tx)
    }

    // TODO: instrument gas price / max_fee_*
    #[instrument(
        follows_from = [process_pending_block_span],
        skip_all,
        fields(
            from = ?tx.from().unwrap(),
            gas = ?tx.gas().unwrap(),
            nonce = ?tx.nonce().unwrap(),
            tx_hash,
        ),
        err,
    )]
    async fn sign_and_send(
        &self,
        tx: TypedTransaction,
        process_pending_block_span: impl Into<Option<tracing::Id>>,
    ) -> anyhow::Result<TxHash> {
        // TODO: debug!("assigned nonce: {nonce}");
        let signature = self.wallet.sign_transaction_sync(&tx)?;
        let raw = tx.rlp_signed(&signature);
        let hash = keccak256(&raw).into();
        Span::current().record("tx_hash", field::debug(hash));
        let pending_tx = self.client.send_raw_transaction(raw).await?;
        if pending_tx.tx_hash() != hash {
            return Err(anyhow!("got wrong pending tx hash after send"));
        }
        info!("transaction sent");
        Ok(hash)
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
