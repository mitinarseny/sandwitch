use std::{collections::HashMap, convert::Infallible, ops::Deref, sync::Arc};

use ethers::{
    providers::{JsonRpcClient, Middleware, Provider, ProviderError},
    signers::Signer,
    types::{
        transaction::eip2718::TypedTransaction, Address, BlockId, Transaction, TxHash, H256, U256,
    },
};
use futures::{
    future::{self, Future, FutureExt, LocalBoxFuture, TryFuture, TryFutureExt},
    lock::{Mutex, OwnedMutexGuard},
    stream::{FusedStream, FuturesUnordered, StreamExt, TryStreamExt},
};
use thiserror::Error;
use tracing::warn;

use crate::cached::{CachedAt, CachedAtBlock};

mod pending_state {
    use std::mem;

    use ethers::types::{BlockNumber, Transaction};
    use futures::future::try_join;
    use metrics::{register_counter, register_gauge, Counter, Gauge};

    use super::{
        Address, JsonRpcClient, Middleware, Provider, ProviderError, TypedTransaction, U256,
    };

    pub(super) struct PendingState {
        last_gas_price: Option<U256>,
        last_pending_gas_price: Gauge, // gauge for last_gas_price, -1 means None

        next_nonce: U256,
        txs_count: Counter, // counter for next_nonce
    }

    impl PendingState {
        // return Ok(None) if account already has some pending transactions
        pub(super) async fn new<P: JsonRpcClient>(
            provider: &Provider<P>,
            address: Address,
        ) -> Result<Option<Self>, ProviderError> {
            let next_nonce = {
                let (next_nonce, pending_count) = try_join(
                    provider.get_transaction_count(address, Some(BlockNumber::Latest.into())),
                    provider.get_transaction_count(address, Some(BlockNumber::Pending.into())),
                )
                .await?;
                if next_nonce != pending_count {
                    return Ok(None);
                }
                next_nonce
            };

            let mut s = Self {
                last_gas_price: None,
                last_pending_gas_price: register_gauge!(
                    "sandwitch_last_pending_gas_price",
                    "address" => format!("{address:x}"),
                ),
                next_nonce: 0.into(),
                txs_count: register_counter!(
                    "sandwitch_txs_count",
                    "address" => format!("{address:x}"),
                ),
            };
            s.set_last_gas_price(None);
            s.set_next_nonce(next_nonce);
            Ok(Some(s))
        }

        pub(super) fn can_send(&self, gas_price: U256) -> bool {
            self.last_gas_price.map_or(true, move |g| g >= gas_price)
        }

        fn alloc_next_nonce(&mut self, gas_price: U256) -> Option<U256> {
            if !self.set_last_gas_price(gas_price) {
                return None;
            }
            self.set_next_nonce(self.next_nonce())
        }

        pub(super) fn assign_next_nonce<'a>(
            &mut self,
            tx: &'a mut TypedTransaction,
        ) -> Option<&'a mut TypedTransaction> {
            self.alloc_next_nonce(tx.gas_price().unwrap())
                .map(move |n| tx.set_nonce(n))
        }

        pub(super) fn tx_mined(&mut self, tx: &Transaction) {
            if self.set_next_nonce(tx.nonce + 1).is_some() {
                self.set_last_gas_price(None);
            }
        }

        fn next_nonce(&self) -> U256 {
            self.next_nonce + 1
        }

        fn set_next_nonce(&mut self, next_nonce: U256) -> Option<U256> {
            if next_nonce < self.next_nonce {
                return None;
            }
            let prev_nonce = mem::replace(&mut self.next_nonce, next_nonce);
            self.txs_count.absolute(self.next_nonce.as_u64());
            Some(prev_nonce)
        }

        fn set_last_gas_price(&mut self, gas_price: impl Into<Option<U256>>) -> bool {
            match gas_price.into() {
                Some(gas_price) => {
                    if !self.can_send(gas_price) {
                        return false;
                    }
                    self.last_gas_price = Some(gas_price);
                    self.last_pending_gas_price.set(gas_price.as_u64() as f64);
                    true
                }
                None => {
                    self.last_gas_price = None;
                    self.last_pending_gas_price.set(-1f64);
                    false
                }
            }
        }
    }
}

use self::pending_state::PendingState;

pub(crate) struct InnerAccount<P: JsonRpcClient, S: Signer> {
    provider: Arc<Provider<P>>,
    signer: S,
    balance: CachedAtBlock<U256>,
}

#[derive(Error, Debug)]
pub(crate) enum SendTxError<S: Signer, E = Infallible> {
    #[error("signer")]
    Sign(S::Error),

    #[error("provider")]
    Provider(ProviderError),

    #[error(transparent)]
    Other(#[from] E),
}

impl<S: Signer, E> SendTxError<S, E> {
    #[allow(dead_code)]
    pub(crate) fn map_other<F, W>(self, f: F) -> SendTxError<S, W>
    where
        F: FnOnce(E) -> W,
    {
        match self {
            Self::Sign(e) => SendTxError::Sign(e),
            Self::Provider(e) => SendTxError::Provider(e),
            Self::Other(e) => SendTxError::Other(f(e)),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn other_into<W>(self) -> SendTxError<S, W>
    where
        E: Into<W>,
    {
        self.map_other(Into::into)
    }
}

impl<S: Signer> SendTxError<S, Infallible> {
    #[allow(dead_code)]
    pub(crate) fn from_never<E>(self) -> SendTxError<S, E> {
        self.map_other(|_| unreachable!())
    }
}

impl<P: JsonRpcClient, S: Signer> InnerAccount<P, S> {
    pub(crate) fn address(&self) -> Address {
        self.signer.address()
    }

    #[allow(dead_code)]
    pub(crate) async fn balance_at(&self, block_hash: H256) -> Result<U256, ProviderError> {
        self.balance
            .get_at_or_try_insert_with(block_hash, |block_hash| {
                self.provider
                    .get_balance(self.address(), BlockId::Hash(*block_hash).into())
            })
            .await
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

pub(crate) struct Account<P: JsonRpcClient, S: Signer> {
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
    // return Ok(None) if account already has some pending transactions
    pub(crate) async fn new(
        provider: impl Into<Arc<Provider<P>>>,
        signer: S,
    ) -> Result<Option<Self>, ProviderError> {
        let provider = provider.into();

        let Some(pending_state) = PendingState::new(&provider, signer.address()).await? else {
            return Ok(None);
        };

        Ok(Some(Self {
            pending_state: Arc::new(Mutex::new(pending_state)),
            inner: Arc::new(InnerAccount {
                provider,
                signer,
                balance: CachedAt::default(),
            }),
        }))
    }

    pub(crate) async fn lock(self) -> LockedAccount<P, S> {
        LockedAccount {
            inner: self.inner,
            pending_state: self.pending_state.lock_owned().await,
        }
    }

    // TODO
    // pub(crate) async fn sync_pending_state(&self, block: BlockId) -> Result<(), ProviderError> {
    //     let mut pending_nonces = self.pending_state.lock().await;
    //     *pending_nonces = self.inner.get_pending_state(block).await?;
    //     Ok(())
    // }
}

pub(crate) struct LockedAccount<P: JsonRpcClient, S: Signer> {
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
    pub(crate) fn tx_mined(&mut self, tx: &Transaction) {
        if tx.from != self.address() {
            return;
        }
        self.pending_state.tx_mined(tx)
    }

    #[allow(dead_code)]
    pub(crate) fn can_send(&self, gas_price: U256) -> bool {
        self.pending_state.can_send(gas_price)
    }

    #[allow(dead_code)]
    pub(crate) fn send_tx(
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

    #[allow(dead_code)]
    pub(crate) fn send_txs(
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

pub(crate) struct Accounts<P: JsonRpcClient, S: Signer>(HashMap<Address, Account<P, S>>);

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
    pub(crate) async fn from_signers(
        signers: impl IntoIterator<Item = S>,
        provider: impl Into<Arc<Provider<P>>>,
    ) -> Result<Self, ProviderError> {
        signers
            .into_iter()
            .map({
                let provider = provider.into();
                move |s| {
                    let address = s.address();
                    Account::new(provider.clone(), s).inspect_ok(move |a| {
                        if a.is_none() {
                            warn!(
                                ?address,
                                "this account already has some pending transactions \
                                    and can not be used, skipping...",
                            );
                        }
                    })
                }
            })
            .collect::<FuturesUnordered<_>>()
            .try_filter_map(future::ok)
            .try_collect()
            .await
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn get(&self, address: &Address) -> Option<Account<P, S>> {
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

    #[allow(dead_code)]
    pub(crate) async fn find<F>(&self, pred: F) -> Option<Account<P, S>>
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

    #[allow(dead_code)]
    pub(crate) async fn try_find<F, E>(&self, pred: F) -> Result<Option<Account<P, S>>, E>
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

    #[allow(dead_code)]
    pub(crate) async fn find_map<F, T>(&self, f: F) -> Option<(Account<P, S>, T)>
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

    #[allow(dead_code)]
    pub(crate) async fn try_find_map<F, T, E>(&self, f: F) -> Result<Option<(Account<P, S>, T)>, E>
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

    #[allow(dead_code)]
    pub(crate) async fn find_locked<F>(&self, pred: F) -> Option<LockedAccount<P, S>>
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

    #[allow(dead_code)]
    pub(crate) async fn try_find_locked<F, E>(
        &self,
        pred: F,
    ) -> Result<Option<LockedAccount<P, S>>, E>
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

    #[allow(dead_code)]
    pub(crate) async fn find_map_locked<F, T>(&self, f: F) -> Option<(LockedAccount<P, S>, T)>
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

    #[allow(dead_code)]
    pub(crate) async fn try_find_map_locked<F, T, E>(
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

    #[allow(dead_code)]
    pub(crate) async fn try_find_wrap_and_send_txs<F, E: 'static>(
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
        {
            let wrap_gas_price = wrap_tx.gas_price.unwrap();

            let Some((mut locked_account, (front_run_txs, back_run_txs))) =
                self.try_find_map_locked(|locked_account| {
                    if !locked_account.can_send(wrap_gas_price - 1) {
                        return future::ok(None).boxed_local()
                    }
                    f(locked_account)
                }).await? else {
                return Ok(None);
            };

            locked_account.send_txs(
                front_run_txs
                    .into_iter()
                    .map(move |mut tx| {
                        tx.set_gas_price(wrap_gas_price - 1);
                        tx
                    })
                    .chain(back_run_txs.into_iter().map(move |mut tx| {
                        tx.set_gas_price(wrap_gas_price + 1);
                        tx
                    })),
            )
            // locked_account is dropped here
        }
        .into_future()
        .await
        .map(Some)
        .map_err(SendTxError::from_never)
    }
}
