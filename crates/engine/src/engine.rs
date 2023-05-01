use core::pin::pin;
use std::sync::Arc;

use anyhow::{anyhow, Context};
use ethers::{
    providers::{JsonRpcClient, Middleware, Provider, ProviderError, PubsubClient},
    signers::{LocalWallet, Signer},
    types::{
        transaction::eip2718::TypedTransaction, Address, Block, BlockNumber, Filter, TxHash, U256,
    },
};
use futures::{
    future::{self, Aborted, Fuse, FusedFuture, Future, FutureExt},
    select_biased,
    stream::{FuturesOrdered, FuturesUnordered, StreamExt, TryStreamExt},
    try_join,
};

// use metrics::{register_counter, register_gauge, register_histogram, Counter, Histogram};

use tokio::{
    self,
    time::{sleep_until, timeout_at, Duration, Instant},
};
use tokio_util::sync::CancellationToken;
use tracing::{
    debug, error, error_span, field, info, info_span, instrument, warn, Instrument, Span,
};

use sandwitch_contracts::{
    multicall::{MultiCall, MultiCallContract},
    EthTypedCall,
};

use crate::{
    abort::FutureExt as AbortFutureExt,
    block::{PendingBlock, PendingBlockFactory, PrioritizedMultiCall},
    config::Config,
    monitor::PendingBlockMonitor,
    providers::LatencyProvider,
    timed::StreamExt as TimedStreamExt,
    transactions::TransactionRequest,
};

// TODO: use Signer Middleware
pub type MiddlewareStack<P> = Provider<LatencyProvider<P>>;

pub struct Engine<P, M>
where
    P: JsonRpcClient,
    M: PendingBlockMonitor<MiddlewareStack<P>>,
{
    client: Arc<MiddlewareStack<P>>,
    multicall: Arc<MultiCallContract<Arc<MiddlewareStack<P>>, MiddlewareStack<P>>>,
    address: Address,
    wallet: Option<LocalWallet>,
    pending_block_factory: PendingBlockFactory<MiddlewareStack<P>>,
    // next_block_at_estimator: NextBlockAtEstimator,
    tx_propagation_delay: Duration, // TODO: move into next block at estimator
    block_interval: Duration,
    monitor: M,
}

// struct OnChainData<M> {
//     multicall: Arc<MultiCallContract<Arc<M>, M>>,

// }

impl<P, M> Engine<P, M>
where
    P: PubsubClient + 'static,
    P::Error: Send + Sync + 'static,
    M: PendingBlockMonitor<MiddlewareStack<P>>,
{
    pub async fn new(
        client: impl Into<Arc<MiddlewareStack<P>>>,
        cfg: Config,
        wallet: impl Into<Option<LocalWallet>>,
        monitor: M,
    ) -> anyhow::Result<Self> {
        let client = client.into();
        let wallet = wallet.into();
        let multicall = Arc::new(MultiCallContract::new(cfg.multicall, client.clone()));

        let (owner, mining) = try_join!(
            future::ok(Address::zero()), // TODO
            // multicall.owner(),
            client.mining(),
        )?;
        if let Some(address) = wallet.as_ref().map(LocalWallet::address) {
            if address != owner {
                return Err(anyhow!(
                    "{multicall:?} is owned by {owner:?}, but the wallet has following address: {address:?}"
                ));
            }
        }
        if !mining {
            warn!("node is not mining");
        }

        Ok(Self {
            client,
            address: owner,
            wallet,
            pending_block_factory: PendingBlockFactory::new(owner, multicall.clone()),
            multicall,
            // next_block_at_estimator: NextBlockAtEstimator::new(cfg.block_interval),
            tx_propagation_delay: cfg.tx_propagation_delay,
            block_interval: cfg.block_interval,
            monitor,
        })
    }

    pub fn account(&self) -> Address {
        self.address
    }

    pub async fn run(self, cancel: CancellationToken) -> anyhow::Result<()> {
        let mut send_txs = FuturesUnordered::new();

        let r = {
            let mut cancelled = pin!(cancel.cancelled().map(|_| Aborted));
            let mut blocks = self
                .client
                .subscribe_blocks()
                .with_abort(&mut cancelled)
                .await?
                .with_context(|| "failed to subscribe to new blocks")?
                .timed()
                .fuse();
            debug!("listening to new blocks");

            let mut process_pending_block = pin!(Fuse::terminated());

            let mut next_block_at_estimator = NextBlockAtEstimator::new(self.block_interval);

            macro_rules! break_err {
                ($result:expr) => {
                    match $result {
                        Ok(v) => v,
                        Err(err) => break Err(err),
                    }
                };
            }
            loop {
                select_biased! {
                    sent_tx_hash = send_txs.select_next_some() => {
                        break_err!(sent_tx_hash);
                    },
                    _ = &mut cancelled => {
                        info!("cancelled");
                        break Ok(());
                    },
                    (block, received_at) = blocks.select_next_some() => Self::new_head_received_span(&block).in_scope(|| {
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


                        let deadline =
                            next_block_at_estimator.estimate_next_block_at(received_at)
                            - self.latency() // reserve time to send txs to the node
                            - self.tx_propagation_delay; // reserve time for txs to propagate through the network

                        process_pending_block.set(
                            // don't send txs the after deadline even if new head hasn't been received
                            timeout_at(
                                deadline,
                                self.process_pending_block(block, deadline).in_current_span(),
                            )
                            .fuse(),
                        );
                    }),
                    to_send = &mut process_pending_block => match to_send {
                        Ok(to_send) => if let Some(to_send) = break_err!(to_send) {
                            // TODO: debug! sending txs...
                            send_txs.extend(to_send);
                        },
                        Err(elapsed) => {
                            warn!("{elapsed}");
                            continue;
                        },
                    },
                    complete => return Ok(()),
                };
            }
        };

        if send_txs.is_empty() {
            return r;
        }
        if let Err(err) = &r {
            error!(%err, "error");
        }

        info!(
            left_to_send = send_txs.len(),
            "still sending transactions, waiting for them to finish...",
        );

        while let Some(sent_tx_hash) = send_txs.next().await {
            if let Err(err) = sent_tx_hash {
                error!(%err, "failed to send transaction");
            }
        }
        r
    }

    fn new_head_received_span(block: &Block<TxHash>) -> Span {
        error_span!(
            "new_head_received",
            block_hash = ?block.hash.unwrap(),
            block_number = ?block.number.unwrap().as_u64(),
            parent_block_hash = ?block.parent_hash,
        )
    }

    #[instrument(skip(self), fields(%block_number))]
    async fn get_next_nonce_at(&self, block_number: BlockNumber) -> anyhow::Result<Option<U256>> {
        let (nonce_at_block, pending_nonce) = try_join!(
            self.client
                .get_transaction_count(self.account(), Some(block_number.into())),
            self.client
                .get_transaction_count(self.account(), Some(BlockNumber::Pending.into())),
        )?;
        if pending_nonce < nonce_at_block {
            return Err(anyhow!(
                "pending txs count can not be greater than at already mined block"
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

    #[instrument(skip(self), fields(%block_number))]
    async fn get_my_balance_at(&self, block_number: BlockNumber) -> anyhow::Result<U256> {
        self.client
            .get_balance(self.account(), Some(block_number.into()))
            .await
            .map_err(Into::into)
    }

    #[instrument(skip_all)]
    async fn get_pending_block(&self) -> anyhow::Result<PendingBlock<'_, MiddlewareStack<P>>> {
        let log_filter = Filter::new().select(BlockNumber::Pending);
        let (block, logs) = try_join!(
            self.client.get_block_with_txs(BlockNumber::Pending),
            self.client.get_logs(&log_filter),
        )?;
        let Some(block) = block else {
            error!("pending block doest not exist");
            return Err(ProviderError::UnsupportedRPC.into());
        };
        self.pending_block_factory
            .new_pending_block(block, logs)
            .await
            .map_err(Into::into)
    }

    fn latency(&self) -> Duration {
        self.client.as_ref().as_ref().latency()
    }

    #[instrument(skip_all, fields(parent_block_hash, block_number,), err)]
    async fn process_pending_block(
        &self,
        latest_block: Block<TxHash>,
        deadline: Instant,
    ) -> anyhow::Result<
        Option<impl Iterator<Item = impl Future<Output = anyhow::Result<TxHash>> + '_> + '_>,
    > {
        let span = Span::current();

        let latest_block_number: BlockNumber = latest_block.number.unwrap().into();

        let (next_nonce, my_balance, pending_block) = try_join!(
            self.get_next_nonce_at(latest_block_number),
            self.get_my_balance_at(latest_block_number),
            async {
                sleep_until(deadline - Duration::from_secs(3))
                    .instrument(info_span!("wait_before_request_pending"))
                    .await;
                debug!("requesting pending block...");
                self.get_pending_block().await
            }
        )?;

        let Some(next_nonce) = next_nonce else {
            return Ok(None);
        };

        // TODO: use my_balance to check if we have enough ETH to pay for gas

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
        // TODO: maybe force sleep until abort_processing_at, so we would send just at the end of the block?
        match timeout_at(
            deadline - self.latency(), // reserve time to estimate gas for all produced txs
            self.monitor.process_pending_block(&pending_block),
        )
        .await
        {
            Ok(v) => v?,
            Err(elapsed) => {
                warn!("{elapsed}");
                return Ok(None);
            }
        }
        debug!("pending block processed");
        // TODO: log to_send count

        let to_send = self.extract_txs_to_send(pending_block, next_nonce).await?;

        let Some(wallet) = &self.wallet else {
            warn!("unable to sign: wallet is not set");
            return Ok(None);
        };

        Ok(Some(
            to_send
                .into_iter()
                .map(|mut tx| {
                    tx.set_chain_id(wallet.chain_id());
                    let signature = wallet.sign_transaction_sync(&tx)?;
                    anyhow::Ok(tx.rlp_signed(&signature))
                    // TODO: debug! signed
                })
                .try_collect::<Vec<_>>()?
                .into_iter()
                .map(move |tx| {
                    async move {
                        let pending_tx = self.client.send_raw_transaction(tx).await?;
                        Ok(pending_tx.tx_hash())
                    } // TODO: instrument from, gas, nonce, tx_hash, gas_price?
                    .instrument(span.clone())
                }),
        ))
    }

    #[instrument(skip_all)]
    async fn extract_txs_to_send(
        &self,
        processed_block: PendingBlock<'_, MiddlewareStack<P>>,
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
            .nonce(nonce);

        #[cfg(not(feature = "legacy"))]
        {
            tx = tx
                .max_priority_fee_per_gas(p.priority_fee_per_gas)
                .max_fee_per_gas(block.base_fee_per_gas.unwrap() + p.priority_fee_per_gas);
        }
        #[cfg(feature = "legacy")]
        {
            tx = tx.gas_price(p.priority_fee_per_gas);
        }

        let mut tx: TypedTransaction = tx.into();
        tx.set_gas(
            self.client
                .estimate_gas(&tx, Some(BlockNumber::Pending.into()))
                .await?,
        );
        Ok(tx)
    }
}

#[derive(Default)]
struct NextBlockAtEstimator {
    block_interval: Duration,
}

impl NextBlockAtEstimator {
    pub fn new(block_interval: Duration) -> Self {
        Self { block_interval }
    }

    pub fn estimate_next_block_at(&mut self, received_at: Instant) -> Instant {
        received_at + self.block_interval
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
