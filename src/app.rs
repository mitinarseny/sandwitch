use std::{str::FromStr, sync::Arc, time::Duration, vec::Vec};

use anyhow::anyhow;
use ethers::{
    core::k256::ecdsa::SigningKey,
    providers::{Ipc, Middleware, Provider, PubsubClient, Ws},
    signers::LocalWallet,
    types::Address,
    utils::secret_key_to_address,
};
use futures::try_join;
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
    engine::{Engine, EngineConfig, ProviderStack},
    monitors::{MultiMonitor, PendingBlockMonitor},
    providers::{LatencyProvider, TimeoutProvider},
};

// #[cfg(feature = "pancake_swap")]
// use crate::monitors::pancake_swap::{PancakeSwap, PancakeSwapConfig};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub network: NetworkConfig,
    pub engine: EngineConfig,
    pub monitors: MonitorsConfig,
}

#[derive(Deserialize, Debug)]
pub struct NetworkConfig {
    pub node: Url,
}

#[derive(Deserialize, Debug)]
pub struct MonitorsConfig {
    // #[cfg(feature = "pancake_swap")]
    // pub pancake_swap: Option<PancakeSwapConfig>,
}

pub struct App<P, M> {
    engine: Engine<P, M>,
}

const CLIENT_TIMEOUT: Duration = Duration::from_secs(5);

impl<P> App<P, Box<dyn PendingBlockMonitor<ProviderStack<P>>>>
where
    P: PubsubClient + 'static,
{
    pub async fn new(client: P, signing_key: SigningKey, cfg: Config) -> anyhow::Result<Self> {
        let client = Arc::new(Provider::new(LatencyProvider::new(TimeoutProvider::new(
            client,
            CLIENT_TIMEOUT,
        ))));
        info!("initializing...");
        let (network_id, chain_id) = try_join!(
            client.get_net_version(),
            client.get_chainid().map_ok(|v| v.as_u64())
        )?;
        info!(network_id, %chain_id, "node info");
        register_counter!(
            "sandwitch_info",
            "network_id" => network_id,
            "chain_id" => format!("{chain_id}"),
        )
        .absolute(1);

        let monitor = App::make_monitor(client.clone(), cfg.monitors).await?;

        Ok(Self {
            engine: Engine::new(
                client,
                cfg.engine,
                {
                    let address = secret_key_to_address(&signing_key);
                    LocalWallet::new_with_signer(signing_key, address, chain_id)
                },
                monitor,
            )
            .await?,
        })
    }

    async fn make_monitor(
        client: impl Into<Arc<Provider<P>>>,
        // accounts: impl Into<Arc<Accounts<P, S>>>,
        config: MonitorsConfig,
    ) -> anyhow::Result<Box<dyn PendingBlockMonitor<Provider<P>>>> {
        let monitors = Self::make_monitors(client, config).await?;
        Ok(match monitors.len() {
            0 => {
                warn!("all monitors are disabled, starting is watch mode...");
                Box::new(())
            }
            1 => monitors.into_iter().next().unwrap(),
            _ => Box::new(MultiMonitor::from_iter(monitors)),
        })
    }

    #[allow(unused_variables)]
    async fn make_monitors(
        client: impl Into<Arc<Provider<P>>>,
        // accounts: impl Into<Arc<Accounts<P, S>>>,
        config: MonitorsConfig,
    ) -> anyhow::Result<Vec<Box<dyn PendingBlockMonitor<Provider<P>>>>> {
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

impl<P, M> App<P, M>
where
    P: PubsubClient + 'static,
    M: PendingBlockMonitor<ProviderStack<P>>,
{
    pub async fn run(self, cancel: CancellationToken) -> anyhow::Result<()> {
        self.engine.run(cancel).await
    }
}
