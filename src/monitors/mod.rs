use ethers::prelude::{Block, Transaction};
use ethers::types::TxHash;
use futures::future::{join_all, BoxFuture};
use futures::FutureExt;

pub mod pancake_swap;

pub trait Monitor<Input> {
    fn process<'a>(&'a mut self, input: &'a Input) -> BoxFuture<'a, ()>;
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
    fn process<'a>(&'a mut self, input: &'a Transaction) -> BoxFuture<'a, ()> {
        join_all(self.0.iter_mut().map(|m| m.process(input)))
            .map(|_| ())
            .boxed()
    }
}

impl Monitor<Block<TxHash>> for MultiTxMonitor {
    fn process<'a>(&'a mut self, input: &'a Block<TxHash>) -> BoxFuture<'a, ()> {
        join_all(self.0.iter_mut().map(|m| m.process(input)))
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
