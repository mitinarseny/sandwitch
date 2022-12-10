use std::{collections::HashSet, sync::Arc};

use anyhow::Context;
use ethers::{
    providers::{JsonRpcClient, Middleware, Provider, PubsubClient},
    signers::Signer,
    types::{Block, Transaction, TxHash, H256},
};
use futures::{
    future::{join, Future, FutureExt, TryFutureExt},
    pin_mut, select_biased,
    stream::{FusedStream, FuturesUnordered, StreamExt, TryStreamExt},
    try_join,
};
use tokio;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, trace, warn};

use crate::{abort::JoinHandleSet, accounts::Accounts, monitors::TxMonitor};

pub struct Engine<SC, RC, S, M>
where
    SC: PubsubClient,
    RC: JsonRpcClient,
    S: Signer,
    M: TopTxMonitor,
{
    streaming: Arc<Provider<SC>>,
    requesting: Arc<Provider<RC>>,
    accounts: Arc<Accounts<RC, S>>,
    monitor: Arc<M>,
}

pub trait TopTxMonitor: TxMonitor<Ok = (), Error = anyhow::Error> + Sync + Send + 'static {}

impl<M> TopTxMonitor for M where M: TxMonitor<Ok = (), Error = anyhow::Error> + Sync + Send + 'static
{}

impl<SC, RC, S, M> Engine<SC, RC, S, M>
where
    SC: PubsubClient + 'static,
    RC: JsonRpcClient + 'static,
    S: Signer + 'static,
    M: TopTxMonitor,
{
    pub fn new(
        streaming: impl Into<Arc<Provider<SC>>>,
        requesting: impl Into<Arc<Provider<RC>>>,
        accounts: impl Into<Arc<Accounts<RC, S>>>,
        monitor: impl Into<Arc<M>>,
    ) -> Self {
        Self {
            accounts: accounts.into(),
            streaming: streaming.into(),
            requesting: requesting.into(),
            monitor: monitor.into(),
        }
    }

    pub async fn run(&mut self, cancel: CancellationToken) -> anyhow::Result<()> {
        let (mut new_blocks, mut pending_txs) = try_join!(
            self.streaming
                .subscribe_blocks()
                .inspect_ok(|st| info!(subscription_id = %st.id, "subscribed to new blocks"))
                .map(|r| r.with_context(|| "failed to subscribe to new blocks")),
            self.streaming
                .subscribe_pending_txs()
                .inspect_ok(
                    |st| info!(subscription_id = %st.id, "subscribed to new pending transactions"),
                )
                .map(|r| r.with_context(|| "failed to subscribe to new pending transactions")),
        )?;

        let txs = pending_txs.by_ref().take_until(cancel.cancelled());
        pin_mut!(txs);
        let mut blocks = new_blocks
            .by_ref()
            // .inspect(|block|) // TODO: upd height and current block_hash
            .map(Ok)
            .try_filter_map(|block| self.requesting.get_block_with_txs(block.hash.unwrap()))
            .fuse();

        let mut handles = JoinHandleSet::default();

        let cancelled = cancel.cancelled().fuse();
        pin_mut!(cancelled);

        let mut first_err = None;
        while !(txs.is_terminated() && handles.is_terminated()) {
            select_biased! {
                _ = cancelled => {
                    handles.abort_all();
                },
                block = blocks.select_next_some() => {
                    tokio::spawn(self.process_block(block?, &mut handles)); // TODO: wait?
                },
                r = handles.select_next_some() => if let Err(err) = r {
                    error!(%err, "transaction processing failed");
                    if first_err.is_none() {
                        cancel.cancel();
                        first_err = Some(err);
                    }
                    // TODO: upd statistics
                },
                tx_hash = txs.select_next_some() => self.maybe_process_tx(tx_hash, &mut handles),
            }
        }

        // manually unsibscribe, since just dropping streams causes panics
        // from separate tokio task, which actually produces these streams
        join(
            pending_txs.unsubscribe().map(|r| match r {
                Err(err) => error!(%err, "failed to unsubscribe from new pending transactions"),
                Ok(_) => info!("unsubscribed from new pending transactions"),
            }),
            new_blocks.unsubscribe().map(|r| match r {
                Err(err) => error!(%err, "failed to unsubscribe from new blocks"),
                Ok(_) => info!("unsubscribed from new blocks"),
            }),
        )
        .await;

        first_err.map_or(Ok(()), Err).map_err(Into::into)
    }

    fn process_block(
        &self,
        block: Block<Transaction>,
        handles: &mut JoinHandleSet<TxHash, anyhow::Result<Option<M::Ok>>>,
    ) -> impl Future<Output = ()> {
        block
            .transactions
            .into_iter()
            .rev()
            .inspect(|tx| {
                handles.abort(&tx.hash);
            })
            .filter_map({
                let mut seen = HashSet::new();
                move |tx| {
                    if let Some(account) = self.accounts.get(&tx.from) {
                        if seen.insert(account.address()) {
                            return Some(account.lock().map(move |mut a| a.nonce_mined(tx.nonce)));
                        }
                    }
                    None
                }
            })
            .collect::<FuturesUnordered<_>>()
            .collect::<()>()
    }

    fn maybe_process_tx(
        &self,
        tx_hash: TxHash,
        handles: &mut JoinHandleSet<TxHash, anyhow::Result<Option<M::Ok>>>,
    ) {
        let handle_entry = match handles.try_insert(tx_hash) {
            Ok(v) => v,
            Err(tx_hash) => {
                trace!(
                    ?tx_hash,
                    "this transaction is already being processed, skipping...",
                );
                return;
            }
        };

        handle_entry.spawn(Self::process_tx(
            self.requesting.clone(),
            self.monitor.clone(),
            tx_hash,
            tx_hash,
        ));
    }

    #[tracing::instrument(skip_all, fields(?tx_hash, ?block_hash))]
    async fn process_tx(
        provider: Arc<Provider<RC>>,
        monitor: Arc<M>,
        tx_hash: H256,
        block_hash: H256,
    ) -> anyhow::Result<Option<M::Ok>> {
        let Some(tx) = provider.get_transaction(tx_hash)
            .await
            .with_context(|| "failed to get transaction")?
            else {
            trace!("fake transaction, skipping...");
            return Ok(None);
        };

        monitor
            .process_tx(&tx, block_hash)
            .await
            .map(Some)
            .map_err(Into::into)
    }
}
