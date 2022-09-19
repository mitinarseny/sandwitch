use std::ops::Deref;

use ethers::prelude::{Block, EthCall, Transaction};
use ethers::types::TxHash;
use futures::TryFutureExt;
use futures::{
    future::{try_join_all, BoxFuture},
    FutureExt,
};

pub mod pancake_swap;

pub trait TxMonitor: Send + Sync {
    fn on_tx<'a>(&'a self, tx: &'a Transaction) -> BoxFuture<'a, anyhow::Result<Vec<Transaction>>>;
}

pub trait BlockMonitor: Send + Sync {
    fn on_block<'a>(&'a self, block: &'a Block<TxHash>) -> BoxFuture<'a, anyhow::Result<()>>;
}

pub trait PendingTxMonitor: TxMonitor + BlockMonitor {}
impl<M> PendingTxMonitor for M where M: TxMonitor + BlockMonitor {}

pub trait FunctionCallMonitor<C: EthCall>: Send + Sync {
    fn on_func<'a>(
        &'a self,
        tx: &'a Transaction,
        inputs: &'a C,
    ) -> BoxFuture<'a, anyhow::Result<Vec<Transaction>>>;
}

impl<M> TxMonitor for Box<M>
where
    M: TxMonitor + ?Sized,
{
    #[inline]
    fn on_tx<'a>(&'a self, tx: &'a Transaction) -> BoxFuture<'a, anyhow::Result<Vec<Transaction>>> {
        TxMonitor::on_tx(&**self, tx)
    }
}

impl<M> BlockMonitor for Box<M>
where
    M: BlockMonitor + ?Sized,
{
    #[inline]
    fn on_block<'a>(&'a self, block: &'a Block<TxHash>) -> BoxFuture<'a, anyhow::Result<()>> {
        BlockMonitor::on_block(&**self, block)
    }
}

pub struct MultiTxMonitor<M>(Vec<M>);

impl<M> MultiTxMonitor<M> {
    pub fn new(monitors: impl IntoIterator<Item = M>) -> Self {
        Self(monitors.into_iter().collect())
    }
}

impl<M> Deref for MultiTxMonitor<M> {
    type Target = <Vec<M> as Deref>::Target;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<M> TxMonitor for MultiTxMonitor<M>
where
    M: TxMonitor,
{
    fn on_tx<'a>(&'a self, tx: &'a Transaction) -> BoxFuture<'a, anyhow::Result<Vec<Transaction>>> {
        try_join_all(self.iter().map(|m| m.on_tx(tx)))
            .map_ok(|v| v.concat())
            .boxed()
    }
}

impl<M> BlockMonitor for MultiTxMonitor<M>
where
    M: BlockMonitor,
{
    fn on_block<'a>(&'a self, block: &'a Block<TxHash>) -> BoxFuture<'a, anyhow::Result<()>> {
        try_join_all(self.iter().map(|m| m.on_block(block)))
            .map_ok(|_| ())
            .boxed()
    }
}
