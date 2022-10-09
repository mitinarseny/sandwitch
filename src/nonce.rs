use std::collections::{HashMap, HashSet, VecDeque};
use std::iter;

use ethers::prelude::*;

#[derive(Debug)]
struct PendingNonces {
    base_nonce: U256,
    pending_gas_prices: VecDeque<U256>,
}

impl PendingNonces {
    fn new(base: impl Into<U256>) -> Self {
        Self {
            pending_gas_prices: VecDeque::new(),
            base_nonce: base.into(),
        }
    }

    fn insert_one(&mut self, tx: &mut TransactionRequest) {
        self.insert_many(Some(tx))
    }

    /// txs must be sorted by descending gas_price
    fn insert_many<'a>(&'a mut self, txs: impl IntoIterator<Item = &'a mut TransactionRequest>) {
        let mut txs = txs.into_iter();
        let pending_gas_prices = self.pending_gas_prices.make_contiguous();
        let mut i = 0;
        for tx in txs.by_ref() {
            let gas_price = tx.gas_price.unwrap();
            i += pending_gas_prices[i..].partition_point(|g| g >= &gas_price);
            let cur_nonce = self.base_nonce + i + 1;
            match pending_gas_prices.get_mut(i) {
                Some(g) => {
                    *g = gas_price;
                    tx.nonce = Some(cur_nonce);
                }
                None => {
                    self.pending_gas_prices
                        .extend(iter::once(tx).chain(txs).enumerate().map(|(i, tx)| {
                            tx.nonce = Some(cur_nonce + i);
                            tx.gas_price.unwrap()
                        }));
                    return;
                }
            };
        }
    }

    fn set_base(&mut self, nonce: impl Into<U256>) {
        let nonce = nonce.into();
        let diff = nonce.saturating_sub(self.base_nonce).as_usize();
        if diff == 0 {
            // TODO: warn about nonce being already seen
            return;
        }
        self.base_nonce = nonce;

        let to = diff.min(self.pending_gas_prices.len());
        if to < diff {
            // TODO: warn that we didn't sent this nonce
        }
        self.pending_gas_prices.drain(..to);
    }

    fn len(&self) -> usize {
        self.pending_gas_prices.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_nonces() {
        let mut pn = PendingNonces::new(0);
        // TODO: base can be -1

        let mut txs = vec![
            TransactionRequest::new().gas_price(100),
            TransactionRequest::new().gas_price(50),
            TransactionRequest::new().gas_price(30),
        ];

        pn.insert_many(txs.iter_mut());
        itertools::assert_equal(
            [1, 2, 3],
            txs.iter().filter_map(|tx| tx.nonce).map(|n| n.as_usize()),
        );

        txs = vec![
            TransactionRequest::new().gas_price(110),
            TransactionRequest::new().gas_price(40),
            TransactionRequest::new().gas_price(20),
            TransactionRequest::new().gas_price(10),
        ];
        pn.insert_many(txs.iter_mut());
        itertools::assert_equal(
            [1, 3, 4, 5],
            txs.iter().filter_map(|tx| tx.nonce).map(|n| n.as_usize()),
        );

        pn.set_base(2); // tx.gas_price at this point is 50

        txs = vec![
            TransactionRequest::new().gas_price(90),
            TransactionRequest::new().gas_price(60),
            TransactionRequest::new().gas_price(5),
        ];
        pn.insert_many(txs.iter_mut());
        itertools::assert_equal(
            [3, 4, 6],
            txs.iter().filter_map(|tx| tx.nonce).map(|n| n.as_usize()),
        );
    }
}

pub struct SingleNonceManager {
    sender: Address,
    pending: PendingNonces,
}

impl SingleNonceManager {
    pub fn new(sender: Address, base_nonce: impl Into<U256>) -> Self {
        Self {
            sender,
            pending: PendingNonces::new(base_nonce),
        }
    }

    pub fn insert_many<'a>(
        &'a mut self,
        txs: impl IntoIterator<Item = &'a mut TransactionRequest>,
    ) {
        self.pending.insert_many(txs)
    }

    pub fn on_block_mined(&mut self, block: &Block<Transaction>) {
        if let Some(&n) = block
            .transactions
            .iter()
            .rev()
            .find_map(|tx| (tx.from == self.sender).then_some(&tx.nonce))
        {
            self.pending.set_base(n)
        }
    }
}

#[derive(Default)]
struct MultiNonceManager(HashMap<Address, PendingNonces>);

impl MultiNonceManager {
    fn add_sender(&mut self, address: Address, base_nonce: U256) {
        self.0
            .entry(address)
            .or_insert_with(move || PendingNonces::new(base_nonce));
    }

    fn insert_many<'a>(&'a mut self, txs: impl IntoIterator<Item = &'a mut Transaction>) {
        // TODO: disctribute across senders, trying not to replace existing txs
        // TODO: bear in mind that each account should have enough money to send these txs
        // and enough tokens associated with these accounts
    }

    fn on_block_mined(&mut self, block: &Block<Transaction>) {
        // TODO: check also for txs which we wrapped
        let mut seen = HashSet::new();

        // iterate from the end of block to catch higher nonces first
        for tx in block.transactions.iter().rev() {
            if let Some(ns) = self.0.get_mut(&tx.from) {
                if !seen.insert(tx.from) {
                    continue;
                }
                ns.set_base(tx.nonce); // TODO: check that this is equal to get_transaction_count
                if seen.len() == self.0.len() {
                    // short-circuit since we have already seen all managed addresses
                    break;
                }
            }
        }
    }
}
