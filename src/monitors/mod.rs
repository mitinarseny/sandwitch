use ethers::prelude::{Block, EthCall, Transaction, TransactionRequest};
use ethers::types::TxHash;
use futures::{future::BoxFuture, stream::FuturesUnordered, FutureExt, TryStreamExt};
use tokio::sync::RwLock;

pub mod pancake_swap;

pub trait StatelessBlockMonitor: Send + Sync {
    fn on_block<'a>(&'a self, block: &'a Block<TxHash>) -> BoxFuture<'a, anyhow::Result<()>>;
}

pub trait BlockMonitor: Send + Sync {
    fn on_block<'a>(&'a mut self, block: &'a Block<TxHash>) -> BoxFuture<'a, anyhow::Result<()>>;
}

impl<M> StatelessBlockMonitor for Box<M>
where
    M: StatelessBlockMonitor + ?Sized,
{
    fn on_block<'a>(&'a self, block: &'a Block<TxHash>) -> BoxFuture<'a, anyhow::Result<()>> {
        self.as_ref().on_block(block)
    }
}

impl<M> StatelessBlockMonitor for RwLock<M>
where
    M: BlockMonitor,
{
    fn on_block<'a>(&'a self, block: &'a Block<TxHash>) -> BoxFuture<'a, anyhow::Result<()>> {
        async move { self.write().await.on_block(block).await }.boxed()
    }
}

impl<M> StatelessBlockMonitor for [M]
where
    M: StatelessBlockMonitor,
{
    fn on_block<'a>(&'a self, block: &'a Block<TxHash>) -> BoxFuture<'a, anyhow::Result<()>> {
        self.iter()
            .map(|m| m.on_block(block))
            .collect::<FuturesUnordered<_>>()
            .try_collect()
            .boxed()
    }
}

impl<M> BlockMonitor for Box<M>
where
    M: BlockMonitor + ?Sized,
{
    fn on_block<'a>(&'a mut self, block: &'a Block<TxHash>) -> BoxFuture<'a, anyhow::Result<()>> {
        self.as_mut().on_block(block)
    }
}

impl<M> BlockMonitor for RwLock<M>
where
    M: BlockMonitor,
{
    fn on_block<'a>(&'a mut self, block: &'a Block<TxHash>) -> BoxFuture<'a, anyhow::Result<()>> {
        self.get_mut().on_block(block)
    }
}

impl<M> BlockMonitor for [M]
where
    M: BlockMonitor,
{
    fn on_block<'a>(&'a mut self, block: &'a Block<TxHash>) -> BoxFuture<'a, anyhow::Result<()>> {
        self.iter_mut()
            .map(|m| m.on_block(block))
            .collect::<FuturesUnordered<_>>()
            .try_collect()
            .boxed()
    }
}

pub trait TxMonitor: Send + Sync {
    fn on_tx<'a>(
        &'a self,
        tx: &'a Transaction,
    ) -> BoxFuture<'a, anyhow::Result<Vec<TransactionRequest>>>;
}

impl<M> TxMonitor for Box<M>
where
    M: TxMonitor + ?Sized,
{
    fn on_tx<'a>(
        &'a self,
        tx: &'a Transaction,
    ) -> BoxFuture<'a, anyhow::Result<Vec<TransactionRequest>>> {
        (**self).on_tx(tx)
    }
}

impl<M> TxMonitor for RwLock<M>
where
    M: TxMonitor,
{
    fn on_tx<'a>(
        &'a self,
        tx: &'a Transaction,
    ) -> BoxFuture<'a, anyhow::Result<Vec<TransactionRequest>>> {
        async move { self.read().await.on_tx(tx).await }.boxed()
    }
}

impl<'l, M> TxMonitor for [M]
where
    M: TxMonitor,
{
    fn on_tx<'a>(
        &'a self,
        tx: &'a Transaction,
    ) -> BoxFuture<'a, anyhow::Result<Vec<TransactionRequest>>> {
        self.iter()
            .map(|m| m.on_tx(tx))
            .collect::<FuturesUnordered<_>>()
            .try_concat()
            .boxed()
    }
}

pub trait Monitor: TxMonitor + StatelessBlockMonitor {}
impl<M> Monitor for M where M: TxMonitor + StatelessBlockMonitor {}

pub trait FunctionCallMonitor<C: EthCall>: Send + Sync {
    fn on_func<'a>(
        &'a self,
        tx: &'a Transaction,
        inputs: C,
    ) -> BoxFuture<'a, anyhow::Result<Vec<TransactionRequest>>>;

    fn on_func_raw<'a>(
        &'a self,
        tx: &'a Transaction,
    ) -> Option<BoxFuture<'a, anyhow::Result<Vec<TransactionRequest>>>> {
        let inputs = C::decode(&tx.input).ok()?;
        Some(self.on_func(tx, inputs))
    }
}
