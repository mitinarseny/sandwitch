// use std::ops::{Deref, DerefMut};

use ethers::prelude::{Block, EthCall, Transaction};
use ethers::types::TxHash;
use futures::TryFutureExt;
use futures::{
    future::{try_join_all, BoxFuture},
    FutureExt,
};
use tokio::sync::RwLock;

pub mod pancake_swap;

pub trait TxMonitor: Send + Sync {
    fn on_tx<'a>(&'a self, tx: &'a Transaction) -> BoxFuture<'a, anyhow::Result<Vec<Transaction>>>;
}

impl<M> TxMonitor for Box<M>
where
    M: TxMonitor + ?Sized,
{
    fn on_tx<'a>(&'a self, tx: &'a Transaction) -> BoxFuture<'a, anyhow::Result<Vec<Transaction>>> {
        (**self).on_tx(tx)
    }
}

impl<M> TxMonitor for RwLock<M>
where
    M: TxMonitor,
{
    fn on_tx<'a>(&'a self, tx: &'a Transaction) -> BoxFuture<'a, anyhow::Result<Vec<Transaction>>> {
        async move { self.read().await.on_tx(tx).await }.boxed()
    }
}

impl<'l, M> TxMonitor for [M]
where
    M: TxMonitor,
{
    fn on_tx<'a>(&'a self, tx: &'a Transaction) -> BoxFuture<'a, anyhow::Result<Vec<Transaction>>> {
        try_join_all(self.iter().map(|m| m.on_tx(tx)))
            .map_ok(|v| v.concat())
            .boxed()
    }
}

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
        try_join_all(self.iter().map(|m| m.on_block(block)))
            .map_ok(|_| ())
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
        try_join_all(self.iter_mut().map(|m| m.on_block(block)))
            .map_ok(|_| ())
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
    ) -> BoxFuture<'a, anyhow::Result<Vec<Transaction>>>;

    fn on_func_raw<'a>(
        &'a self,
        tx: &'a Transaction,
    ) -> Option<BoxFuture<'a, anyhow::Result<Vec<Transaction>>>> {
        let inputs = C::decode(&tx.input).ok()?;
        Some(self.on_func(tx, inputs))
    }
}
