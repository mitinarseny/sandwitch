use core::mem;
use std::collections::{hash_map::Entry, HashMap, HashSet};

use async_trait::async_trait;
use ethers::{
    abi::{ethereum_types::BloomInput, AbiDecode, RawLog},
    contract::{EthEvent, EthLogDecode},
    providers::Middleware,
    types::{Address, H160, H256, U256},
};
use hex_literal::hex;
use lazy_static::lazy_static;
use sandwitch_contracts::{
    erc20,
    multicall::{Call, ContractCall, MustCall, TryCall},
    pancake_swap, pancake_toaster,
};
use sandwitch_engine::{
    block::{AdjacentPendingTxsWithLogs, PendingBlock},
    monitor::BlockMonitor,
    transactions::Transaction,
};

pub struct Monitor {
    pancake_router: Address,
    pancake_toaster: Address,
}

lazy_static! {
    static ref PANCAKE_PAIR_SWAP_SIGNATURE: H256 = pancake_swap::pair::SwapFilter::signature();
}

#[async_trait]
impl<M> BlockMonitor<M> for Monitor
where
    M: Middleware,
{
    async fn process_pending_block(&self, block: &PendingBlock<M>) -> anyhow::Result<()> {
        if let Some(b) = block.logs_bloom {
            if !b.contains_input(BloomInput::Hash(&PANCAKE_PAIR_SWAP_SIGNATURE.0)) {
                return Ok(());
            }
        }

        for adjacent_txs in block.iter_adjacent_txs() {
            // TODO: we shouldn't add again pair after it was deleted
            // TODO: add direction of swap
            let mut pools = PoolStates::default();
            for tx in &adjacent_txs {
                if let Some(swap) = self.tx_to_swap(&tx) {
                    for l in tx.logs.iter() {
                        let Ok(event) = pancake_swap::pair::PancakePairEvents::decode_log(
                        &l.clone().into()) else {
                        continue;
                    };
                        match event {
                            pancake_swap::pair::PancakePairEvents::SyncFilter(sync) => {
                                pools.on_sync(l.address, sync);
                            }
                            pancake_swap::pair::PancakePairEvents::SwapFilter(swap) => {
                                pools.on_swap(l.address, swap);
                            }
                            _ => continue,
                        }

                        let Some(swap) = self.tx_to_swap(&tx) else {
                        pools.non_playable(l.address);
                        // TODO: someone touched pool, but not using router, we need to ignore it and don't play on it
                        continue;
                    };

                        // TODO: calculate slippage and update min()
                    }
                }
            }
            let playable_pools = pools.into_playable();
            // TODO: filter out unprofitable, and those which do not start with BASE_TOKEN
            let (front_runs, back_runs): (Vec<_>, Vec<_>) = playable_pools
                .map(|(pool, reserves)| {
                    (
                        ContractCall::new(
                            self.pancake_toaster,
                            pancake_toaster::FrontRunSwapCall {
                                from: block.account(),
                                amount_in: 0.into(), // TODO
                                amount_out: 0.into(), // TODO
                                                     // amount_in:
                            },
                        )
                        .must()
                        .into_dyn(),
                        ContractCall::new(
                            self.pancake_toaster,
                            pancake_toaster::BackRunSwapAllCall {
                            // token_in
                        },
                        )
                        .must()
                        .into_dyn(),
                    )
                })
                .unzip();

            block
                .add_to_send([
                    adjacent_txs.front_run(front_runs),
                    adjacent_txs.back_run(back_runs),
                ])
                .await;
            // TODO: calculate price impact on all these pools and sandwich them if profitable
        }
        Ok(())
    }
}

impl Monitor {
    fn tx_to_swap(&self, tx: &Transaction) -> Option<Swap> {
        if !tx.to.is_some_and(|to| to == self.pancake_router) {
            return None;
        }
        Swap::decode_from_tx(tx)
    }

    async fn process_adjacent_txs<M: Middleware>(
        &self,
        txs: AdjacentPendingTxsWithLogs<'_, M>,
    ) -> anyhow::Result<()> {
        let mut playable_pools: HashMap<Address, (Reserves, Reserves)> = HashMap::new();
        for tx in &txs {
            if let Some(swap) = self.decode_swap_from_tx(&tx) {
            } else {
                for l in &tx.logs {
                    let contract = l.address;
                    if pancake_swap::pair::SwapFilter::decode_log(&l.clone().into()).is_ok() {
                        playable_pools.remove(&contract);
                    }
                }
            }
            for (pool, pool_swap) in tx.logs.iter().cloned().filter_map(|l| {
                let address = l.address;
                let swap_event = pancake_swap::pair::SwapFilter::decode_log(&l.into()).ok()?;
                Some((address, swap_event))
            }) {
                if !tx.to.is_some_and(|to| to == PANCAKE_ROUTER_ADDR) {
                    playable_pools.remove(&pool);
                    continue;
                }
            }
        }
        Ok(())
    }

    fn decode_swap_from_tx(&self, tx: &Transaction) -> Option<Swap> {
        if !tx.to.is_some_and(|to| to == PANCAKE_ROUTER_ADDR) {
            return None;
        }
        Swap::decode_from_tx(tx)
    }
}

struct PlayablePoolReserves {
    before: pancake_swap::pair::SyncFilter,
    after: pancake_swap::pair::SyncFilter,
}

impl PlayablePoolReserves {
    fn from_sync_and_swap(
        sync: pancake_swap::pair::SyncFilter,
        swap: pancake_swap::pair::SwapFilter,
    ) -> Self {
        Self {
            before: todo!(), // TODO: subtract swap from after
            after: sync,
        }
    }

    fn sync(&mut self, sync: pancake_swap::pair::SyncFilter) {
        self.after = sync;
    }
}

enum PoolState {
    FirstSynced(pancake_swap::pair::SyncFilter),
    Swapped(PlayablePoolReserves),
}

impl PoolState {
    fn first_sync(sync: pancake_swap::pair::SyncFilter) -> Self {
        Self::FirstSynced(sync)
    }

    fn swap(&mut self, swap: pancake_swap::pair::SwapFilter) {
        if let Self::FirstSynced(sync) = *self {
            *self = Self::Swapped(PlayablePoolReserves::from_sync_and_swap(sync, swap));
        }
    }

    fn sync(&mut self, sync: pancake_swap::pair::SyncFilter) -> bool {
        let Self::Swapped(swapped) = self else {
            return false;
        };
        swapped.sync(sync);
        true
    }

    fn into_playable(self) -> Option<PlayablePoolReserves> {
        match self {
            Self::Swapped(playable) => Some(playable),
            Self::FirstSynced(_) => None,
        }
    }
}

#[derive(Default)]
struct PoolStates(HashMap<Address, Option<PoolState>>);

impl PoolStates {
    fn on_sync(&mut self, contract: Address, sync: pancake_swap::pair::SyncFilter) {
        match self.0.entry(contract) {
            Entry::Occupied(mut entry) => {
                let Some(pool) = entry.get_mut() else {
                    return;
                };
                if !pool.sync(sync) {
                    entry.remove();
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(Some(PoolState::first_sync(sync)));
            }
        }
    }

    fn on_swap(&mut self, contract: Address, swap: pancake_swap::pair::SwapFilter) {
        let Some(state) = self.0.entry(contract).or_default().as_mut() else {
            return;
        };
        state.swap(swap);
    }

    fn non_playable(&mut self, contract: Address) {
        self.0.insert(contract, None);
    }

    fn into_playable(self) -> impl Iterator<Item = (Address, PlayablePoolReserves)> {
        self.0
            .into_iter()
            .filter_map(|(address, state)| Some((address, state?.into_playable()?)))
    }
}

struct Reserves {
    reserve_in: U256,
    reserve_out: U256,
}

pub type TokenBalances = HashMap<Address, U256>;

#[async_trait(?Send)]
pub trait DEXMonitor {
    async fn deltas(&self, balances: &mut TokenBalances) -> anyhow::Result<HashMap<Address, ()>>;
}

pub trait KPool {
    const FEE: u8;
    fn pair(tokenA: Address, tokenB: Address) -> Address;
}

struct Swap {
    pub amount_in: U256,
    pub amount_out: U256,
    pub eth_in: bool,
    pub path: Vec<Address>,
}

impl Swap {
    fn decode_from_tx(tx: &Transaction) -> Option<Self> {
        let inputs = PancakeRouterCalls::decode(&tx.input).ok()?;
        Some(match inputs {
            PancakeRouterCalls::SwapExactETHForTokens(s) => Self {
                amount_in: tx.value,
                amount_out: s.amount_out_min,
                eth_in: true,
                path: s.path,
            },
            PancakeRouterCalls::SwapETHForExactTokens(s) => Self {
                amount_in: tx.value,
                amount_out: s.amount_out,
                eth_in: true,
                path: s.path,
            },
            PancakeRouterCalls::SwapExactTokensForTokens(s) => Self {
                amount_in: s.amount_in,
                amount_out: s.amount_out_min,
                eth_in: false,
                path: s.path,
            },
            PancakeRouterCalls::SwapTokensForExactTokens(s) => Self {
                amount_in: s.amount_in_max,
                amount_out: s.amount_out,
                eth_in: false,
                path: s.path,
            },
            PancakeRouterCalls::SwapExactTokensForETH(s) => Self {
                amount_in: s.amount_in,
                amount_out: s.amount_out_min,
                eth_in: false,
                path: s.path,
            },
            PancakeRouterCalls::SwapTokensForExactETH(s) => Self {
                amount_in: s.amount_in_max,
                amount_out: s.amount_out,
                eth_in: false,
                path: s.path,
            },
            _ => return None,
        })
    }
}
