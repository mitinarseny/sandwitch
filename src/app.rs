use std::{sync::Arc, time::Duration, vec::Vec};

use ethers::{
    core::k256::ecdsa::SigningKey,
    providers::{Middleware, Provider, PubsubClient, Ws},
    signers::LocalWallet,
    types::Address,
};
#[allow(unused_imports)]
use futures::{
    future::{self, FutureExt, LocalBoxFuture, TryFutureExt},
    stream::{FuturesUnordered, TryStreamExt},
};
use metrics::register_counter;
use serde::Deserialize;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};
use url::Url;

use crate::{
    // accounts::Accounts,
    engine::Engine,
    monitors::{MultiMonitor, PendingBlockMonitor},
    timeout::TimeoutProvider,
};

// #[cfg(feature = "pancake_swap")]
// use crate::monitors::pancake_swap::{PancakeSwap, PancakeSwapConfig};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub engine: EngineConfig,
    pub monitors: MonitorsConfig,
}

#[derive(Deserialize, Debug)]
pub struct EngineConfig {
    pub wss: Url,
    pub chain_id: u64,
    pub multicall: Address,
}

#[derive(Deserialize, Debug)]
pub struct MonitorsConfig {
    // #[cfg(feature = "pancake_swap")]
    // pub pancake_swap: Option<PancakeSwapConfig>,
}

const TIMEOUT: Duration = Duration::from_secs(5);

pub struct App<P>
where
    P: PubsubClient + 'static,
    P::Error: Send + Sync + 'static,
{
    engine: Engine<P, Box<dyn PendingBlockMonitor>>,
}

impl App<Ws> {
    pub async fn from_config(config: Config, signing_key: SigningKey) -> anyhow::Result<Self> {
        let client = Arc::new(Provider::new(TimeoutProvider::new(
            Ws::connect(config.engine.wss).await?,
            TIMEOUT,
        )));
        info!("web socket created");

        let network_id = client.get_net_version().await?;
        info!(network_id);
        register_counter!("sandwitch_info", "network_id" => network_id).absolute(1);

        // let accounts_dir = accounts_dir.as_ref();
        // let keys = Self::read_keys(accounts_dir).await?;
        // if keys.is_empty() {
        //     warn!("no keys found in {}", accounts_dir.display());
        // } else {
        //     info!(
        //         count = keys.len(),
        //         "keys collected, initializing accounts..."
        //     );
        // }

        // let accounts = Arc::new(Accounts::from_signers(keys, client.clone()).await?);
        // info!(count = accounts.len(), "accounts initialized");

        let monitor = App::make_monitor(client.clone(), config.monitors).await?;

        Ok(Self {
            engine: Engine::new(
                client,
                LocalWallet::new_with_signer(signing_key, Address::zero(), config.engine.chain_id),
                monitor,
            ),
        })
    }
}

impl<P> App<P>
where
    P: PubsubClient + 'static,
    P::Error: Send + Sync,
{
    pub async fn run(self, cancel: CancellationToken) -> anyhow::Result<()> {
        self.engine.run(cancel).await
    }

    async fn make_monitor(
        client: impl Into<Arc<Provider<P>>>,
        // accounts: impl Into<Arc<Accounts<P, S>>>,
        config: MonitorsConfig,
    ) -> anyhow::Result<Box<dyn PendingBlockMonitor>> {
        let monitors = Self::make_monitors(client, config).await?;
        Ok(match monitors.len() {
            0 => {
                warn!("all monitors are disabled, starting is watch mode...");
                Box::new(())
            }
            1 => monitors.into_iter().next().unwrap(),
            _ => Box::new(MultiMonitor::from(monitors)),
        })
    }

    #[allow(unused_variables)]
    async fn make_monitors(
        client: impl Into<Arc<Provider<P>>>,
        // accounts: impl Into<Arc<Accounts<P, S>>>,
        config: MonitorsConfig,
    ) -> anyhow::Result<Vec<Box<dyn PendingBlockMonitor>>> {
        let client = client.into();
        let monitors = FuturesUnordered::<LocalBoxFuture<_>>::new();

        // #[cfg(feature = "pancake_swap")]
        // if let Some(cfg) = config.pancake_swap {
        //     tx_monitors.push(
        //         PancakeSwap::from_config(client.clone(), accounts, cfg)
        //             .map_ok(|m| Box::new(m) as Box<dyn TopTxMonitor>)
        //             .boxed_local(),
        //     );
        // }

        monitors.try_collect().await
    }
}
