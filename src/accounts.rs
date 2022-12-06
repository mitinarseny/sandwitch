use anyhow::Context;
use ethers::types::transaction::eip2718::TypedTransaction;
use futures::future::{self, join, BoxFuture, LocalBoxFuture};
use futures::lock::{Mutex, MutexGuard, OwnedMutexGuard};
use futures::stream::{self, FusedStream, FuturesUnordered, StreamExt};
use futures::{Future, FutureExt, Stream, TryFuture, TryFutureExt, TryStreamExt};
use std::collections::HashMap;
use std::ops::Deref;
use std::panic;
use std::sync::Arc;

use ethers::prelude::*;

pub type TxGroup = Vec<TransactionRequest>;

struct PendingState {
    last_gas_price: Option<U256>,
    next_nonce: U256,
    // TODO: metrics per account + balance
}

impl PendingState {
    fn can_send(&self, gas_price: U256) -> bool {
        self.last_gas_price.map_or(true, |g| g <= gas_price)
    }

    fn next_nonce(&mut self, gas_price: U256) -> U256 {
        if self.last_gas_price.map_or(false, |g| g > gas_price) {
            panic!(); // TODO
        }
        self.last_gas_price = Some(gas_price);
        let nonce = self.next_nonce;
        self.next_nonce += 1;
        nonce
    }
}

pub struct InnerAccount<P: JsonRpcClient, S: Signer> {
    provider: Arc<Provider<P>>,
    signer: S,
}

impl<P: JsonRpcClient, S: Signer> InnerAccount<P, S> {
    pub fn address(&self) -> Address {
        self.signer.address()
    }

    async fn get_pending_state(
        &self,
        block: impl Into<Option<BlockId>>,
    ) -> Result<PendingState, ProviderError> {
        let next_nonce = self
            .provider
            .get_transaction_count(self.address(), block.into())
            .await?;
        // TODO: count pending nonces and gas prices???
        Ok(PendingState {
            last_gas_price: None,
            next_nonce,
        })
    }

    async fn send_tx(&self, tx: TypedTransaction) -> anyhow::Result<TxHash> {
        let tx = tx.rlp_signed(
            &self
                .signer
                .sign_transaction(&tx)
                .await
                .with_context(|| "failed to sign transaction")?,
        );
        let ptx = self
            .provider
            .send_raw_transaction(tx)
            .await
            .with_context(|| "failed to send raw transaction")?;
        Ok(ptx.tx_hash())
    }

    pub async fn balance(&self, block: impl Into<Option<BlockId>>) -> Result<U256, ProviderError> {
        todo!()
    }
}

#[derive(Clone)]
pub struct Account<P: JsonRpcClient, S: Signer> {
    inner: Arc<InnerAccount<P, S>>,
    pending_state: Arc<Mutex<PendingState>>,
}

impl<P: JsonRpcClient, S: Signer> Deref for Account<P, S> {
    type Target = InnerAccount<P, S>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<P: JsonRpcClient, S: Signer> Account<P, S> {
    pub async fn new(
        provider: impl Into<Arc<Provider<P>>>,
        signer: S,
    ) -> Result<Self, ProviderError> {
        let inner = InnerAccount {
            provider: provider.into(),
            signer,
        };
        let pending_state = inner.get_pending_state(None).await?;
        Ok(Self {
            pending_state: Arc::new(Mutex::new(pending_state)),
            inner: Arc::new(inner),
        })
    }

    pub async fn lock(&self) -> LockedAccount<P, S> {
        LockedAccount {
            inner: self.inner.clone(),
            pending_state: self.pending_state.clone().lock_owned().await,
        }
    }

    pub async fn sync_pending_state(
        &self,
        block: impl Into<Option<BlockId>>,
    ) -> Result<(), ProviderError> {
        let mut pending_nonces = self.pending_state.lock().await;
        *pending_nonces = self.inner.get_pending_state(block).await?;
        Ok(())
    }
}

pub struct LockedAccount<P: JsonRpcClient, S: Signer> {
    inner: Arc<InnerAccount<P, S>>,
    pending_state: OwnedMutexGuard<PendingState>,
}

impl<P: JsonRpcClient, S: Signer> Deref for LockedAccount<P, S> {
    type Target = InnerAccount<P, S>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<P: JsonRpcClient, S: Signer> LockedAccount<P, S> {
    pub fn can_send(&self, gas_price: U256) -> bool {
        self.pending_state.can_send(gas_price)
    }

    fn set_next_nonce(&mut self, tx: &mut TypedTransaction) -> &mut TypedTransaction {
        tx.set_nonce(self.pending_state.next_nonce(tx.gas_price().unwrap()))
    }

    pub fn send_tx(
        &mut self,
        tx: TypedTransaction,
    ) -> impl TryFuture<Ok = TxHash, Error = anyhow::Error> {
        self.set_next_nonce(&mut tx);
        tokio::spawn(self.inner.send_tx(tx)).map(Result::unwrap)
    }

    pub fn send_txs(
        &mut self,
        txs: impl IntoIterator<Item = TypedTransaction>,
    ) -> impl TryFuture<Ok = Vec<TxHash>, Error = anyhow::Error> {
        let txs = txs.into_iter().map({
            |tx| {
                self.set_next_nonce(&mut tx);
                tx
            }
        });

        // spawn as a separate task to ensure that allocated nonces
        // were actually sent to the network in case of someone drops
        // this future
        tokio::spawn(
            txs.map({
                let inner = self.inner.clone();
                move |tx| inner.send_tx(tx)
            })
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>(),
        )
        .map(Result::unwrap)
    }
}

#[derive(Default)]
pub struct Accounts<P: JsonRpcClient, S: Signer>(HashMap<Address, Account<P, S>>);

impl<P: JsonRpcClient, S: Signer, A: Into<Arc<Account<P, S>>>> Extend<A> for Accounts<P, S> {
    fn extend<T: IntoIterator<Item = A>>(&mut self, iter: T) {
        self.0
            .extend(iter.into_iter().map(Into::into).map(|a| (a.address(), a)))
    }
}

impl<P: JsonRpcClient, S: Signer> Accounts<P, S> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    fn map_unordered<F, Fut>(&self, f: F) -> impl FusedStream<Fut::Output>
    where
        F: Fn(Account<P, S>) -> Fut,
        Fut: Future,
    {
        self.0
            .values()
            .cloned()
            .map(f)
            .collect::<FuturesUnordered<_>>()
    }

    fn map_unordered_locked(&self, f: F) -> impl FusedStream<Fut::Output>
    where
        F: Fn(LockedAccount<P, S>) -> Fut,
        Fut: Future,
    {
        self.map_unordered(|account| async move { account.lock().map(f) })
    }

    pub async fn find<F, Fut>(&self, pred: F) -> Option<Account<P, S>>
    where
        F: Fn(&Account<P, S>) -> Fut,
        Fut: Future<Output = bool>,
    {
        self.map_unordered(|account| async move { pred(&account).await.then_some(account) })
            .filter_map(future::ready)
            .next()
            .await
    }

    pub async fn try_find<F, Fut>(&self, pred: F) -> Result<Option<Account<P, S>>, Fut::Error>
    where
        F: Fn(&Account<P, S>) -> Fut,
        Fut: TryFuture<Ok = bool>,
    {
        self.0
            .map_unordered(|account| async move {
                pred(&account).into_future().await?.then_some(account)
            })
            .try_filter_map(future::ready)
            .try_next()
            .await
    }

    pub async fn find_map<F, Fut, R>(&self, f: F) -> Option<(Account<P, S>, R)>
    where
        F: Fn(&Account<P, S>) -> Fut,
        Fut: Future<Output = Option<R>>,
    {
        self.map_unordered(|account| async move { f(&account).await.map(move |r| (account, r)) })
            .filter_map(future::ready)
            .next()
            .await
    }

    pub async fn try_find_map<F, Fut, R>(
        &self,
        f: F,
    ) -> Result<Option<(Account<P, S>, R)>, Fut::Error>
    where
        F: Fn(&Account<P, S>) -> Fut,
        Fut: TryFuture<Ok = Option<R>>,
    {
        self.map_unordered(|account| async move {
            f(&account).into_future().await?.map(move |r| (account, r))
        })
        .try_filter_map(future::ready)
        .try_next()
        .await
    }

    pub async fn find_locked<F, Fut>(&self, pred: F) -> Option<LockedAccount<P, S>>
    where
        F: Fn(&LockedAccount<P, S>) -> Fut,
        Fut: Future<Output = bool>,
    {
        self.map_unordered_locked(|locked_account| async move {
            pred(&locked_account).await.then_some(locked_account)
        })
        .filter_map(future::ready)
        .next()
        .await
    }

    pub async fn try_find_locked<F, Fut>(
        &self,
        pred: F,
    ) -> Result<Option<LockedAccount<P, S>>, Fut::Error>
    where
        F: Fn(&LockedAccount<P, S>) -> Fut,
        Fut: TryFuture<Ok = bool>,
    {
        self.map_unordered_locked(|locked_account| async move {
            pred(&locked_account).await?.then_some(locked_account)
        })
        .try_filter_map(future::ready)
        .try_next()
        .await
    }

    pub async fn find_map_locked<F, Fut, R>(&self, f: F) -> Option<(LockedAccount<P, S>, R)>
    where
        F: Fn(&LockedAccount<P, S>) -> Fut,
        Fut: Future<Output = Option<R>>,
    {
        self.map_unordered_locked(|locked_account| async move {
            f(&locked_account).await.map(move |r| (locked_account, r))
        })
        .filter_map(future::ready)
        .next()
        .await
    }

    pub async fn try_find_map_locked<F, Fut, R>(
        &self,
        f: F,
    ) -> Result<Option<(LockedAccount<P, S>, R)>, Fut::Error>
    where
        F: Fn(&LockedAccount<P, S>) -> Fut,
        Fut: TryFuture<Ok = Option<R>>,
    {
        self.map_unordered_locked(|locked_account| async move {
            f(&locked_account)
                .into_future()
                .await?
                .map(move |r| (locked_account, r))
        })
        .try_filter_map(future::ready)
        .try_next()
        .await
    }

    pub async fn find_wrap(&self, wrap_tx: &Transaction) -> Option<LockedAccount<P, S>> {
        let wrap_gas_price = wrap_tx.gas_price.unwrap();
        self.find_locked(|locked_account| {
            future::ready(locked_account.can_send(wrap_gas_price - 1))
        })
        .await
    }

    pub async fn find_wrap_and<F, Fut>(
        &self,
        wrap_tx: &Transaction,
        pred: F,
    ) -> Option<LockedAccount<P, S>>
    where
        F: Fn(&LockedAccount<P, S>) -> Fut,
        Fut: Future<Output = bool>,
    {
        let wrap_gas_price = wrap_tx.gas_price.unwrap();
        self.find_locked(|locked_account| async move {
            locked_account.can_send(wrap_gas_price - 1) && pred(locked_account).await
        })
        .await
    }

    pub async fn try_find_wrap_and<F, Fut>(
        &self,
        wrap_tx: &Transaction,
        pred: F,
    ) -> Result<Option<LockedAccount<P, S>>, Fut::Error>
    where
        F: Fn(&LockedAccount<P, S>) -> Fut,
        Fut: TryFuture<Ok = bool>,
    {
        let wrap_gas_price = wrap_tx.gas_price.unwrap();
        self.try_find_locked(|locked_account| async move {
            if !locked_account.can_send(wrap_gas_price - 1) {
                return Ok(false);
            }
            pred(locked_account).into_future().await
        })
    }

    pub async fn find_wrap_map<F, Fut, R>(
        &self,
        wrap_tx: &Transaction,
        f: F,
    ) -> Option<(LockedAccount<P, S>, R)>
    where
        F: Fn(&LockedAccount<P, S>) -> Fut,
        Fut: Future<Output = Option<R>>,
    {
        let wrap_gas_price = wrap_tx.gas_price.unwrap();
        self.find_map_locked(|locked_account| async move {
            if !locked_account.can_send(wrap_gas_price - 1) {
                return None;
            }
            f(locked_account).await
        })
    }

    pub async fn try_find_wrap_map<F, Fut, R>(
        &self,
        wrap_tx: &Transaction,
        f: F,
    ) -> Result<Option<(LockedAccount<P, S>, R)>, Fut::Error>
    where
        F: Fn(&LockedAccount<P, S>) -> Fut,
        Fut: TryFuture<Output = Option<R>>,
    {
        let wrap_gas_price = wrap_tx.gas_price.unwrap();
        self.find_map_locked(|locked_account| async move {
            if !locked_account.can_send(wrap_gas_price - 1) {
                return Ok(None);
            }
            f(locked_account).into_future().await
        })
    }

    pub async fn try_find_wrap_and_send_txs<F, Fut>(
        &self,
        wrap_tx: &Transaction,
        f: F,
    ) -> anyhow::Result<Vec<TxHash>>
    where
        F: Fn(&LockedAccount<P, S>) -> Fut,
        Fut: TryFuture<Ok = Option<(Vec<TypedTransaction>, Vec<TypedTransaction>)>>,
    {
        let Some((locked_account, (front_run_txs, back_run_txs))) =
            self.try_find_wrap_map(wrap_tx, f).await? else {
            return Ok(None);
        };

        let r = locked_account.send_txs(
            front_run_txs
                .into_iter()
                .map(|tx| {
                    tx.set_gas_price(wrap_gas_price - 1);
                    tx
                })
                .chain(back_run_txs.into_iter().map(|tx| {
                    tx.set_gas_price(wrap_gas_price + 1);
                    tx
                })),
        );
        drop(locked_account); // we do not need lock anymore
        r.into_future().await
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn pending_nonces() {
//         let mut pn = PendingNonces::new(0);
//         // TODO: base can be -1
//
//         let mut txs = vec![
//             TransactionRequest::new().gas_price(100),
//             TransactionRequest::new().gas_price(50),
//             TransactionRequest::new().gas_price(30),
//         ];
//
//         pn.insert_many(txs.iter_mut());
//         itertools::assert_equal(
//             [1, 2, 3],
//             txs.iter().filter_map(|tx| tx.nonce).map(|n| n.as_usize()),
//         );
//
//         txs = vec![
//             TransactionRequest::new().gas_price(110),
//             TransactionRequest::new().gas_price(40),
//             TransactionRequest::new().gas_price(20),
//             TransactionRequest::new().gas_price(10),
//         ];
//         pn.insert_many(txs.iter_mut());
//         itertools::assert_equal(
//             [1, 3, 4, 5],
//             txs.iter().filter_map(|tx| tx.nonce).map(|n| n.as_usize()),
//         );
//
//         pn.set_base(2); // tx.gas_price at this point is 50
//
//         txs = vec![
//             TransactionRequest::new().gas_price(90),
//             TransactionRequest::new().gas_price(60),
//             TransactionRequest::new().gas_price(5),
//         ];
//         pn.insert_many(txs.iter_mut());
//         itertools::assert_equal(
//             [3, 4, 6],
//             txs.iter().filter_map(|tx| tx.nonce).map(|n| n.as_usize()),
//         );
//     }
// }
//
// pub struct SingleNonceManager {
//     sender: Address,
//     pending: PendingNonces,
// }
//
// impl SingleNonceManager {
//     pub fn new(sender: Address, base_nonce: impl Into<U256>) -> Self {
//         Self {
//             sender,
//             pending: PendingNonces::new(base_nonce),
//         }
//     }
//
//     pub fn insert_many<'a>(
//         &'a mut self,
//         txs: impl IntoIterator<Item = &'a mut TransactionRequest>,
//     ) {
//         self.pending.insert_many(txs)
//     }
//
//     pub fn on_block_mined(&mut self, block: &Block<Transaction>) {
//         if let Some(&n) = block
//             .transactions
//             .iter()
//             .rev()
//             .find_map(|tx| (tx.from == self.sender).then_some(&tx.nonce))
//         {
//             self.pending.set_base(n)
//         }
//     }
// }
//
// #[derive(Default)]
// struct MultiNonceManager(HashMap<Address, PendingNonces>);
//
// impl MultiNonceManager {
//     fn add_sender(&mut self, address: Address, base_nonce: U256) {
//         self.0
//             .entry(address)
//             .or_insert_with(move || PendingNonces::new(base_nonce));
//     }
//
//     fn insert_many<'a>(&'a mut self, txs: impl IntoIterator<Item = &'a mut Transaction>) {
//         // TODO: disctribute across senders, trying not to replace existing txs
//         // TODO: bear in mind that each account should have enough money to send these txs
//         // and enough tokens associated with these accounts
//     }
//
//     fn on_block_mined(&mut self, block: &Block<Transaction>) {
//         // TODO: check also for txs which we wrapped
//         let mut seen = HashSet::new();
//
//         // iterate from the end of block to catch higher nonces first
//         for tx in block.transactions.iter().rev() {
//             if let Some(ns) = self.0.get_mut(&tx.from) {
//                 if !seen.insert(tx.from) {
//                     continue;
//                 }
//                 ns.set_base(tx.nonce); // TODO: check that this is equal to get_transaction_count
//                 if seen.len() == self.0.len() {
//                     // short-circuit since we have already seen all managed addresses
//                     break;
//                 }
//             }
//         }
//     }
// }
