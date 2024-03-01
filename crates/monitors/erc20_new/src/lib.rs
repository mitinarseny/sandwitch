#![feature(is_some_and, array_windows)]

use std::{
    collections::{
        hash_map::{self, Entry},
        HashMap, HashSet,
    },
    sync::Arc,
};

use async_trait::async_trait;
use ethers::{
    abi::AbiDecode,
    contract::{ContractError, EthLogDecode},
    providers::Middleware,
    types::Address,
};
use sandwitch_contracts::{
    multicall::{Call, Calls, ContractCall, TryCall},
    pancake_swap::{
        pair::{PancakePairEvents, SwapFilter, SyncFilter},
        router::PancakeRouterCalls,
    },
    pancake_toaster::PancakeToaster,
};
use sandwitch_engine::{
    block::{PendingBlock, TxWithLogs},
    monitor::BlockMonitor,
    transactions::Transaction,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument};

use self::swap::Swap;

// mod factory;
// mod pair;
mod swap;

#[derive(Debug, Serialize, Deserialize)]
pub struct PancakeConfig {
    pub router: Address,
    pub toaster: Address,
    pub base_token: Address,
}

pub struct PancakeMonitor {
    router: Address,
    factory: Address,
    toaster: Address,
    base_token: Address,
}

impl PancakeMonitor {
    #[instrument(skip_all)]
    pub async fn from_config<M>(
        client: Arc<M>,
        cfg: PancakeConfig,
    ) -> Result<Self, ContractError<M>>
    where
        M: Middleware + 'static,
    {
        let router =
            sandwitch_contracts::pancake_swap::router::PancakeRouter::new(cfg.router, client);
        let factory = router.factory().await?;
        info!(?factory);
        Ok(Self {
            router: cfg.router,
            factory,
            toaster: cfg.toaster,
            base_token: cfg.base_token,
        })
    }
}

#[async_trait]
impl<M> BlockMonitor<M> for PancakeMonitor
where
    M: Middleware,
{
    async fn process_pending_block(&self, block: &PendingBlock<M>) -> anyhow::Result<()> {
        for adjacent_txs in block.iter_adjacent_txs() {
            let mut swaps = SwapsTracker::new(self.router, self.factory);
            for tx in &adjacent_txs {
                swaps.on_tx(tx);
            }

            let (front_run_calls, back_run_calls): (Calls<_>, Calls<_>) = swaps
                .into_independent()
                .filter_map(|s| {
                    let index_in = s.path.iter().position(|token| *token == self.base_token)?;
                    if index_in == s.path.len() - 1 {
                        return None;
                    }
                    let back_run = sandwitch_contracts::pancake_toaster::BackRunSwapAllCall {
                        token_in: s.path[index_in],
                        token_out: s.path[index_in + 1],
                    };
                    Some((
                        ContractCall::new(
                            self.toaster,
                            sandwitch_contracts::pancake_toaster::FrontRunSwapCall {
                                from: s.from,
                                amount_in: s.amount_in,
                                amount_out: s.amount_out,
                                eth_in: s.eth_in,
                                path: s.path,
                                index_in: index_in.into(),
                            },
                        )
                        .maybe()
                        .into_dyn(),
                        ContractCall::new(self.toaster, back_run).maybe().into_dyn(),
                    ))
                })
                .unzip();

            block
                .add_to_send([
                    adjacent_txs.front_run(front_run_calls),
                    adjacent_txs.back_run(back_run_calls),
                ])
                .await;
        }

        Ok(())
    }
}

// impl<M> PancakeMonitor<M> {
//     fn tx_to_swap(&self, tx: &Transaction) -> Option<Swap<M>> {
//         if !tx.to.is_some_and(|to| to == self.router) {
//             return None;
//         }
//         let inputs = PancakeRouterCalls::decode(&tx.input).ok()?;
//         Some(match inputs {
//             PancakeRouterCalls::SwapExactETHForTokens(s) => Swap {
//                 from: tx.from,
//                 amount_in: tx.value,
//                 amount_out: s.amount_out_min,
//                 eth_in: true,
//                 pairs: self.path_to_pairs(&s.path).collect(),
//                 path: s.path,
//             },
//             PancakeRouterCalls::SwapETHForExactTokens(s) => Swap {
//                 from: tx.from,
//                 amount_in: tx.value,
//                 amount_out: s.amount_out,
//                 eth_in: true,
//                 pairs: self.path_to_pairs(&s.path).collect(),
//                 path: s.path,
//             },
//             PancakeRouterCalls::SwapExactTokensForTokens(s) => Swap {
//                 from: tx.from,
//                 amount_in: s.amount_in,
//                 amount_out: s.amount_out_min,
//                 eth_in: false,
//                 pairs: self.path_to_pairs(&s.path).collect(),
//                 path: s.path,
//             },
//             PancakeRouterCalls::SwapTokensForExactTokens(s) => Swap {
//                 from: tx.from,
//                 amount_in: s.amount_in_max,
//                 amount_out: s.amount_out,
//                 eth_in: false,
//                 pairs: self.path_to_pairs(&s.path).collect(),
//                 path: s.path,
//             },
//             PancakeRouterCalls::SwapExactTokensForETH(s) => Swap {
//                 from: tx.from,
//                 amount_in: s.amount_in,
//                 amount_out: s.amount_out_min,
//                 eth_in: false,
//                 pairs: self.path_to_pairs(&s.path).collect(),
//                 path: s.path,
//             },
//             PancakeRouterCalls::SwapTokensForExactETH(s) => Swap {
//                 from: tx.from,
//                 amount_in: s.amount_in_max,
//                 amount_out: s.amount_out,
//                 eth_in: false,
//                 pairs: self.path_to_pairs(&s.path).collect(),
//                 path: s.path,
//             },
//             _ => return None,
//         })
//     }

//     fn path_to_pairs(&self, path: &[Address]) -> impl Iterator<Item = (Address, PancakePair<M>)> {
//         path.array_windows().map(|&[from, to]| {
//             let pair = self.factory.get_pair(from, to);
//             (pair.address(), pair)
//         })
//     }
// }

#[derive(Debug)]
pub struct SwapsTracker {
    router: Address,
    factory: Address,
    swaps: Vec<Swap>,
    pools_seen_by_tx_count: HashMap<Address, usize>,
}

impl SwapsTracker {
    pub fn new(router: Address, factory: Address) -> Self {
        Self {
            router,
            factory,
            swaps: Default::default(),
            pools_seen_by_tx_count: Default::default(),
        }
    }

    pub fn on_tx(&mut self, tx: &TxWithLogs) {
        if let Some(swap) = self.tx_to_swap(tx) {
            self.swaps.push(swap);
        }
        for l in &tx.logs {
            let Ok(event) = PancakePairEvents::decode_log(
                &l.clone().into()) else {
                continue;
            };
            match event {
                PancakePairEvents::SyncFilter(sync) => {}
                PancakePairEvents::SwapFilter(swap) => {}
                _ => continue,
            }
            *self.pools_seen_by_tx_count.entry(l.address).or_default() += 1;
        }
    }

    pub fn into_independent(self) -> impl Iterator<Item = Swap> {
        // let pools_seen_by_tx_count = self.pools_seen_by_tx_count;
        self.swaps.into_iter().filter_map(move |swap| {
            swap.pairs()
                .all(|pair| {
                    self.pools_seen_by_tx_count
                        .get(&pair)
                        .copied()
                        .unwrap_or_default()
                        <= 1
                })
                .then_some(swap)
        })
    }

    fn tx_to_swap(&self, tx: &Transaction) -> Option<Swap> {
        if !tx.to.is_some_and(|to| to == self.router) {
            return None;
        }
        info!(tx_hash = ?tx.hash, "to the router");
        let inputs = PancakeRouterCalls::decode(&tx.input).ok()?;
        Some(match inputs {
            PancakeRouterCalls::SwapExactETHForTokens(s) => Swap {
                from: tx.from,
                amount_in: tx.value,
                amount_out: s.amount_out_min,
                eth_in: true,
                path: s.path,
            },
            PancakeRouterCalls::SwapETHForExactTokens(s) => Swap {
                from: tx.from,
                amount_in: tx.value,
                amount_out: s.amount_out,
                eth_in: true,
                path: s.path,
            },
            PancakeRouterCalls::SwapExactTokensForTokens(s) => Swap {
                from: tx.from,
                amount_in: s.amount_in,
                amount_out: s.amount_out_min,
                eth_in: false,
                path: s.path,
            },
            PancakeRouterCalls::SwapTokensForExactTokens(s) => Swap {
                from: tx.from,
                amount_in: s.amount_in_max,
                amount_out: s.amount_out,
                eth_in: false,
                path: s.path,
            },
            PancakeRouterCalls::SwapExactTokensForETH(s) => Swap {
                from: tx.from,
                amount_in: s.amount_in,
                amount_out: s.amount_out_min,
                eth_in: false,
                path: s.path,
            },
            PancakeRouterCalls::SwapTokensForExactETH(s) => Swap {
                from: tx.from,
                amount_in: s.amount_in_max,
                amount_out: s.amount_out,
                eth_in: false,
                path: s.path,
            },
            _ => return None,
        })
    }
}

// pub struct Swaps {
//     factory: Address,
//     router: Address,
//     toched_pools: HashSet<Address>,
//     swaps: Vec<Swap>,
// }

// impl Swaps {
//     pub fn new(factory: Address, router: Address) -> Self {
//         Self {
//             factory,
//             router,
//             toched_pools: Default::default(),
//             swaps: Default::default(),
//         }
//     }

//     pub fn on_tx(&mut self, tx: &TxWithLogs) {
//         let swap = self.tx_to_swap(tx);
//         for l in tx.logs {
//             let Ok(event) = PancakePairEvents::decode_log(
//                 &l.clone().into()) else {
//                 continue;
//             };
//             match event {
//                 PancakePairEvents::SyncFilter(sync) => {
//                     // pools.on_sync(l.address, sync);
//                 }
//                 PancakePairEvents::SwapFilter(swap) => {
//                     // pools.on_swap(l.address, swap);
//                 }
//                 _ => continue,
//             }
//             if !self.toched_pools.insert(l.address) {
//                 // TODO: remove all self.swaps with this pair in path
//             }
//         }
//     }

//     // fn on_swap(&mut self, swap: Swap) {

//     // }

//     // fn on_non_swap(&mut self, swap: Swap) {

//     // }

//     fn tx_to_swap(&self, tx: &Transaction) -> Option<Swap> {
//         if !tx.to.is_some_and(|to| to == self.router) {
//             return None;
//         }
//         let inputs = PancakeRouterCalls::decode(&tx.input).ok()?;
//         Some(match inputs {
//             PancakeRouterCalls::SwapExactETHForTokens(s) => Swap {
//                 from: tx.from,
//                 amount_in: tx.value,
//                 amount_out: s.amount_out_min,
//                 eth_in: true,
//                 path: s.path,
//             },
//             PancakeRouterCalls::SwapETHForExactTokens(s) => Swap {
//                 from: tx.from,
//                 amount_in: tx.value,
//                 amount_out: s.amount_out,
//                 eth_in: true,
//                 path: s.path,
//             },
//             PancakeRouterCalls::SwapExactTokensForTokens(s) => Swap {
//                 from: tx.from,
//                 amount_in: s.amount_in,
//                 amount_out: s.amount_out_min,
//                 eth_in: false,
//                 path: s.path,
//             },
//             PancakeRouterCalls::SwapTokensForExactTokens(s) => Swap {
//                 from: tx.from,
//                 amount_in: s.amount_in_max,
//                 amount_out: s.amount_out,
//                 eth_in: false,
//                 path: s.path,
//             },
//             PancakeRouterCalls::SwapExactTokensForETH(s) => Swap {
//                 from: tx.from,
//                 amount_in: s.amount_in,
//                 amount_out: s.amount_out_min,
//                 eth_in: false,
//                 path: s.path,
//             },
//             PancakeRouterCalls::SwapTokensForExactETH(s) => Swap {
//                 from: tx.from,
//                 amount_in: s.amount_in_max,
//                 amount_out: s.amount_out,
//                 eth_in: false,
//                 path: s.path,
//             },
//             _ => return None,
//         })
//     }

//     pub fn into_independent(self) -> impl Iterator<Item = Swap> {
//         self.swaps.into_iter()
//     }
// }

// #[derive(Default)]
// struct PoolStates(HashMap<Address, Option<PoolState>>);

// impl PoolStates {
//     fn on_sync(&mut self, contract: Address, sync: SyncFilter) {
//         match self.0.entry(contract) {
//             Entry::Occupied(mut entry) => {
//                 let Some(pool) = entry.get_mut() else {
//                     return;
//                 };
//                 if !pool.sync(sync) {
//                     entry.remove();
//                 }
//             }
//             Entry::Vacant(entry) => {
//                 entry.insert(Some(PoolState::first_sync(sync)));
//             }
//         }
//     }

//     fn on_swap(&mut self, contract: Address, swap: SwapFilter) {
//         let Some(state) = self.0.entry(contract).or_default().as_mut() else {
//             return;
//         };
//         state.swap(swap);
//     }

//     fn non_playable(&mut self, contract: Address) {
//         self.0.insert(contract, None);
//     }

//     fn into_playable(self) -> impl Iterator<Item = (Address, PlayablePoolReserves)> {
//         self.0
//             .into_iter()
//             .filter_map(|(address, state)| Some((address, state?.into_playable()?)))
//     }
// }

// #[derive(Debug, Default)]
// pub struct PlayablePools(HashMap<Address, PlayablePoolReserves>);

// impl IntoIterator for PlayablePools {
//     type Item = (Address, PlayablePoolReserves);

//     type IntoIter = hash_map::IntoIter<Address, PlayablePoolReserves>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.0.into_iter()
//     }
// }

// impl Extend<PoolStates> for PlayablePools {
//     fn extend<T: IntoIterator<Item = PoolStates>>(&mut self, iter: T) {
//         for states in iter {
//             for (pool_address, state) in states.0 {
//                 if let Some(state) = state.and_then(PoolState::into_playable) {
//                     self.0.insert(pool_address, state);
//                 } else {
//                     self.0.remove(&pool_address);
//                 }
//             }
//         }
//     }
// }

// struct PlayablePoolReserves {
//     before: SyncFilter,
//     after: SyncFilter,
// }

// impl PlayablePoolReserves {
//     fn from_sync_and_swap(sync: SyncFilter, swap: SwapFilter) -> Self {
//         Self {
//             before: SyncFilter {
//                 reserve_0: sync.reserve_0 + swap.amount_0_in - swap.amount_0_out,
//                 reserve_1: sync.reserve_1 + swap.amount_1_in - swap.amount_1_out,
//             },
//             after: sync,
//         }
//     }

//     fn sync(&mut self, sync: SyncFilter) {
//         self.after = sync;
//     }
// }

// enum PoolState {
//     FirstSynced(Option<SyncFilter>),
//     Swapped(PlayablePoolReserves),
// }

// impl PoolState {
//     fn first_sync(sync: SyncFilter) -> Self {
//         Self::FirstSynced(Some(sync))
//     }

//     fn swap(&mut self, swap: SwapFilter) {
//         if let Self::FirstSynced(sync) = self {
//             *self = Self::Swapped(PlayablePoolReserves::from_sync_and_swap(
//                 sync.take().unwrap(),
//                 swap,
//             ));
//         }
//     }

//     fn sync(&mut self, sync: SyncFilter) -> bool {
//         let Self::Swapped(swapped) = self else {
//             return false;
//         };
//         swapped.sync(sync);
//         true
//     }

//     fn into_playable(self) -> Option<PlayablePoolReserves> {
//         match self {
//             Self::Swapped(playable) => Some(playable),
//             Self::FirstSynced(_) => None,
//         }
//     }
// }
