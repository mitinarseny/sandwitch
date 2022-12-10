use std::{collections::HashMap, convert::Infallible, ops::Deref, panic, sync::Arc};

use ethers::{
    providers::{JsonRpcClient, Middleware, Provider, ProviderError},
    signers::Signer,
    types::{
        transaction::eip2718::TypedTransaction, Address, BlockId, Transaction, TxHash, H256, U256,
    },
};
use futures::{
    future::{self, LocalBoxFuture},
    lock::{Mutex, OwnedMutexGuard},
    stream::{FusedStream, FuturesUnordered, StreamExt},
    Future, FutureExt, TryFuture, TryFutureExt, TryStreamExt,
};
use thiserror::Error;

use crate::cached::{CachedAt, CachedAtBlock};

struct PendingState {
    last_gas_price: Option<U256>,
    next_nonce: U256,
    // TODO: metrics per account + balance
}

impl PendingState {
    fn can_send(&self, gas_price: U256) -> bool {
        self.last_gas_price.map_or(true, |g| g <= gas_price)
    }

    fn alloc_next_nonce(&mut self, gas_price: U256) -> U256 {
        if self.last_gas_price.map_or(false, |g| g > gas_price) {
            panic!(); // TODO
        }
        self.last_gas_price = Some(gas_price);
        let nonce = self.next_nonce;
        self.next_nonce += 1.into();
        nonce
    }

    fn assign_next_nonce<'a, 'b: 'a>(
        &'a mut self,
        tx: &'b mut TypedTransaction,
    ) -> &'b mut TypedTransaction {
        tx.set_nonce(self.alloc_next_nonce(tx.gas_price().unwrap()))
    }

    fn nonce_mined(&mut self, nonce_mined: U256) {
        let new_next_nonce = nonce_mined + 1;
        if new_next_nonce >= self.next_nonce {
            self.next_nonce = new_next_nonce;
            self.last_gas_price = None;
        }
    }
}

pub struct InnerAccount<P: JsonRpcClient, S: Signer> {
    provider: Arc<Provider<P>>,
    signer: S,
    balance: CachedAtBlock<U256>,
}

#[derive(Error, Debug)]
pub enum SendTxError<S: Signer, E = Infallible> {
    #[error("signer")]
    Sign(S::Error),

    #[error("provider")]
    Provider(ProviderError),

    #[error(transparent)]
    Other(#[from] E),
}

impl<S: Signer> SendTxError<S, Infallible> {
    pub fn other_into<E>(self) -> SendTxError<S, E> {
        match self {
            SendTxError::Sign(e) => SendTxError::Sign(e),
            SendTxError::Provider(e) => SendTxError::Provider(e),
            SendTxError::Other(_) => unreachable!(),
        }
    }
}

impl<P: JsonRpcClient, S: Signer> InnerAccount<P, S> {
    pub fn address(&self) -> Address {
        self.signer.address()
    }

    pub async fn balance_at(&self, block_hash: H256) -> Result<U256, ProviderError> {
        self.balance
            .get_at_or_try_insert_with(block_hash, |block_hash| {
                self.provider
                    .get_balance(self.address(), BlockId::Hash(*block_hash).into())
            })
            .await
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

    async fn send_tx(&self, tx: TypedTransaction) -> Result<TxHash, SendTxError<S>> {
        let tx = tx.rlp_signed(
            &self
                .signer
                .sign_transaction(&tx)
                .await
                .map_err(SendTxError::Sign)?,
        );
        let ptx = self
            .provider
            .send_raw_transaction(tx)
            .await
            .map_err(SendTxError::Provider)?;
        Ok(ptx.tx_hash())
    }
}

pub struct Account<P: JsonRpcClient, S: Signer> {
    inner: Arc<InnerAccount<P, S>>,
    pending_state: Arc<Mutex<PendingState>>,
}

impl<P: JsonRpcClient, S: Signer> Clone for Account<P, S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            pending_state: self.pending_state.clone(),
        }
    }
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
            balance: CachedAt::default(),
        };
        let pending_state = inner.get_pending_state(None).await?;
        Ok(Self {
            pending_state: Arc::new(Mutex::new(pending_state)),
            inner: Arc::new(inner),
        })
    }

    pub async fn lock(self) -> LockedAccount<P, S> {
        LockedAccount {
            inner: self.inner,
            pending_state: self.pending_state.lock_owned().await,
        }
    }

    pub async fn sync_pending_state(&self, block: BlockId) -> Result<(), ProviderError> {
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
    // TODO: non-pub?
    pub fn nonce_mined(&mut self, nonce_mined: U256) {
        self.pending_state.nonce_mined(nonce_mined)
    }

    pub fn can_send(&self, gas_price: U256) -> bool {
        self.pending_state.can_send(gas_price)
    }

    pub fn send_tx(
        &mut self,
        mut tx: TypedTransaction,
    ) -> impl TryFuture<Ok = TxHash, Error = SendTxError<S>>
    where
        P: 'static,
        S: 'static,
    {
        self.pending_state.assign_next_nonce(&mut tx);
        tokio::spawn({
            let inner = self.inner.clone();
            async move { inner.send_tx(tx).await }
        })
        .map(Result::unwrap)
    }

    pub fn send_txs(
        &mut self,
        txs: impl IntoIterator<Item = TypedTransaction>,
    ) -> impl TryFuture<Ok = Vec<TxHash>, Error = SendTxError<S>>
    where
        P: 'static,
        S: 'static,
    {
        let txs = txs.into_iter().map({
            |mut tx| {
                self.pending_state.assign_next_nonce(&mut tx);
                tx
            }
        });

        // spawn as a separate task to ensure that allocated nonces
        // were actually sent to the network in case of someone drops
        // this future
        tokio::spawn(
            txs.map(|tx| {
                let inner = self.inner.clone();
                async move { inner.send_tx(tx).await }
            })
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>(),
        )
        .map(Result::unwrap)
    }
}

pub struct Accounts<P: JsonRpcClient, S: Signer>(HashMap<Address, Account<P, S>>);

impl<P: JsonRpcClient, S: Signer> Extend<Account<P, S>> for Accounts<P, S> {
    fn extend<T: IntoIterator<Item = Account<P, S>>>(&mut self, iter: T) {
        self.0.extend(iter.into_iter().map(|a| (a.address(), a)))
    }
}

impl<P: JsonRpcClient, S: Signer> Default for Accounts<P, S> {
    fn default() -> Self {
        Self(HashMap::default())
    }
}

impl<P: JsonRpcClient, S: Signer> Accounts<P, S> {
    pub async fn from_signers(
        signers: impl IntoIterator<Item = S>,
        provider: impl Into<Arc<Provider<P>>>,
    ) -> Result<Self, ProviderError> {
        signers
            .into_iter()
            .map({
                let provider = provider.into();
                move |s| Account::new(provider.clone(), s)
            })
            .collect::<FuturesUnordered<_>>()
            .try_collect()
            .await
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get(&self, address: &Address) -> Option<Account<P, S>> {
        self.0.get(&address).cloned()
    }

    fn map_unordered<'a, F, Fut>(&'a self, f: F) -> impl FusedStream<Item = Fut::Output> + 'a
    where
        F: FnMut(&'a Account<P, S>) -> Fut,
        Fut: Future + 'a,
    {
        self.0.values().map(f).collect::<FuturesUnordered<_>>()
    }

    fn map_unordered_locked<'a, F, Fut>(&'a self, f: F) -> impl FusedStream<Item = Fut::Output> + 'a
    where
        F: Fn(LockedAccount<P, S>) -> Fut + Clone + 'a,
        Fut: Future + 'a,
    {
        self.map_unordered(move |account| account.clone().lock().then(f.clone()))
    }

    pub async fn find<F>(&self, pred: F) -> Option<Account<P, S>>
    where
        F: for<'b> Fn(&'b Account<P, S>) -> LocalBoxFuture<'b, bool>,
    {
        self.map_unordered(|account| {
            let pred = &pred;
            async move { pred(&account).await.then_some(account) }
        })
        .filter_map(future::ready)
        .next()
        .await
        .cloned()
    }

    pub async fn try_find<F, E>(&self, pred: F) -> Result<Option<Account<P, S>>, E>
    where
        F: for<'b> Fn(&'b Account<P, S>) -> LocalBoxFuture<'b, Result<bool, E>>,
    {
        self.map_unordered(|account| {
            let pred = &pred;
            async move { Ok(pred(&account).await?.then_some(account)) }
        })
        .try_filter_map(future::ok)
        .try_next()
        .await
        .map(|o| o.cloned())
    }

    pub async fn find_map<F, T>(&self, f: F) -> Option<(Account<P, S>, T)>
    where
        F: for<'b> Fn(&'b Account<P, S>) -> LocalBoxFuture<'b, Option<T>>,
    {
        self.map_unordered(|account| {
            let f = &f;
            async move { f(&account).await.map(move |r| (account.clone(), r)) }
        })
        .filter_map(future::ready)
        .next()
        .await
    }

    pub async fn try_find_map<F, T, E>(&self, f: F) -> Result<Option<(Account<P, S>, T)>, E>
    where
        F: for<'b> Fn(&'b Account<P, S>) -> LocalBoxFuture<'b, Result<Option<T>, E>>,
    {
        self.map_unordered(|account| {
            let f = &f;
            async move { Ok(f(&account).await?.map(move |r| (account.clone(), r))) }
        })
        .try_filter_map(future::ok)
        .try_next()
        .await
    }

    pub async fn find_locked<F>(&self, pred: F) -> Option<LockedAccount<P, S>>
    where
        F: for<'b> Fn(&'b LockedAccount<P, S>) -> LocalBoxFuture<'b, bool>,
    {
        self.map_unordered_locked(|locked_account| {
            let pred = &pred;
            async move { pred(&locked_account).await.then_some(locked_account) }
        })
        .filter_map(future::ready)
        .next()
        .await
    }

    pub async fn try_find_locked<F, E>(&self, pred: F) -> Result<Option<LockedAccount<P, S>>, E>
    where
        F: for<'b> Fn(&'b LockedAccount<P, S>) -> LocalBoxFuture<'b, Result<bool, E>>,
    {
        self.map_unordered_locked(|locked_account| {
            let pred = &pred;
            async move { Ok(pred(&locked_account).await?.then_some(locked_account)) }
        })
        .try_filter_map(future::ok)
        .try_next()
        .await
    }

    pub async fn find_map_locked<F, T>(&self, f: F) -> Option<(LockedAccount<P, S>, T)>
    where
        F: for<'b> Fn(&'b LockedAccount<P, S>) -> LocalBoxFuture<'b, Option<T>>,
    {
        self.map_unordered_locked(|locked_account| {
            let f = &f;
            async move { f(&locked_account).await.map(move |r| (locked_account, r)) }
        })
        .filter_map(future::ready)
        .next()
        .await
    }

    pub async fn try_find_map_locked<F, T, E>(
        &self,
        f: F,
    ) -> Result<Option<(LockedAccount<P, S>, T)>, E>
    where
        F: (for<'b> Fn(&'b LockedAccount<P, S>) -> LocalBoxFuture<'b, Result<Option<T>, E>>),
    {
        self.map_unordered_locked(|locked_account| {
            let f = &f;
            async move { Ok(f(&locked_account).await?.map(move |r| (locked_account, r))) }
        })
        .try_filter_map(future::ok)
        .try_next()
        .await
    }

    pub async fn try_find_wrap_and_send_txs<F, E: 'static>(
        &self,
        wrap_tx: &Transaction,
        f: F,
    ) -> Result<Option<Vec<TxHash>>, SendTxError<S, E>>
    where
        F: for<'b> Fn(
            &'b LockedAccount<P, S>,
        ) -> LocalBoxFuture<
            'b,
            Result<Option<(Vec<TypedTransaction>, Vec<TypedTransaction>)>, E>,
        >,
        P: 'static,
        S: 'static,
    {
        let wrap_gas_price: U256 = wrap_tx.gas_price.unwrap();

        let Some((mut locked_account, (front_run_txs, back_run_txs))) =
            self.try_find_map_locked(|locked_account| {
                if !locked_account.can_send(wrap_gas_price - 1) {
                    return future::ok(None).boxed_local()
                }
                f(locked_account)
            }).await? else {
            return Ok(None);
        };

        let r = locked_account.send_txs(
            front_run_txs
                .into_iter()
                .map(|mut tx| {
                    tx.set_gas_price(wrap_gas_price - 1);
                    tx
                })
                .chain(back_run_txs.into_iter().map(|mut tx| {
                    tx.set_gas_price(wrap_gas_price + 1);
                    tx
                })),
        );
        drop(locked_account); // we do not need lock anymore
        r.into_future()
            .await
            .map(Some)
            .map_err(SendTxError::other_into)
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
