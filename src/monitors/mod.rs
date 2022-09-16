use ethers::prelude::{Block, Transaction};
use ethers::types::TxHash;
use futures::future::{join_all, BoxFuture};
use futures::FutureExt;

pub mod pancake_swap;

pub trait Monitor: Send + Sync {
    fn on_tx<'a>(&'a self, tx: &'a Transaction) -> BoxFuture<'a, Vec<Transaction>>;

    fn on_block<'a>(&'a self, block: &'a Block<TxHash>) -> BoxFuture<'a, ()>;
}

pub struct MultiTxMonitor(Vec<Box<dyn Monitor>>);

impl MultiTxMonitor {
    pub fn new(monitors: Vec<Box<dyn Monitor>>) -> Self {
        Self(monitors)
    }
}

impl Monitor for MultiTxMonitor {
    fn on_tx<'a>(&'a self, tx: &'a Transaction) -> BoxFuture<'a, Vec<Transaction>> {
        join_all(self.0.iter().map(|m| m.on_tx(tx)))
            .map(|v| v.concat())
            .boxed()
    }

    fn on_block<'a>(&'a self, block: &'a Block<TxHash>) -> BoxFuture<'a, ()> {
        join_all(self.0.iter().map(|m| m.on_block(block)))
            .map(|_| ())
            .boxed()
    }
}
