use ethers::prelude::{Block, Transaction};
use ethers::types::TxHash;
use futures::future::{join_all, BoxFuture};
use futures::FutureExt;

pub mod pancake_swap;

pub trait Monitor<Input> {
    fn process(&mut self, input: Input) -> BoxFuture<'_, ()>;
}

pub trait TxMonitor: Monitor<Transaction> + Monitor<Block<TxHash>> {
    fn flush(&mut self) -> Vec<Transaction>;
}

pub struct MultiTxMonitor(Vec<Box<dyn TxMonitor + Send>>);

impl MultiTxMonitor {
    pub fn new(monitors: Vec<Box<dyn TxMonitor + Send>>) -> Self {
        Self(monitors)
    }
}

impl Monitor<Transaction> for MultiTxMonitor {
    fn process(&mut self, input: Transaction) -> BoxFuture<'_, ()> {
        join_all(self.0.iter_mut().map(|m| m.process(input.clone())))
            .map(|_| ())
            .boxed()
    }
}

impl Monitor<Block<TxHash>> for MultiTxMonitor {
    fn process(&mut self, input: Block<TxHash>) -> BoxFuture<'_, ()> {
        join_all(self.0.iter_mut().map(|m| m.process(input.clone())))
            .map(|_| ())
            .boxed()
    }
}

impl TxMonitor for MultiTxMonitor {
    fn flush(&mut self) -> Vec<Transaction> {
        self.0
            .iter_mut()
            .map(|m| m.flush())
            .reduce(|mut v1, v2| {
                v1.extend(v2.into_iter());
                v1
            })
            .unwrap_or_else(|| Vec::new())
    }
}
