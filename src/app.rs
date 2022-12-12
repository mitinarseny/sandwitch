use std::{path::Path, sync::Arc, vec::Vec};

use anyhow::anyhow;
use ethers::{
    core::k256::ecdsa::SigningKey,
    providers::{Middleware, Provider, PubsubClient, Ws},
    signers::{Signer, Wallet},
};
use futures::{
    future::{self, FutureExt, LocalBoxFuture, TryFutureExt},
    stream::{FuturesUnordered, TryStreamExt},
};
use metrics::register_counter;
use serde::Deserialize;
use tokio::fs;
use tokio_stream::wrappers::ReadDirStream;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};
use url::Url;

use crate::{
    accounts::Accounts,
    engine::{Engine, TopTxMonitor},
    monitors::TxMonitorExt,
};

#[cfg(feature = "pancake_swap")]
use crate::monitors::pancake_swap::{PancakeSwap, PancakeSwapConfig};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub engine: EngineConfig,
    pub monitors: MonitorsConfig,
}

#[derive(Deserialize, Debug)]
pub struct EngineConfig {
    pub wss: Url,
}

#[derive(Deserialize, Debug)]
pub struct MonitorsConfig {
    #[cfg(feature = "pancake_swap")]
    pub pancake_swap: Option<PancakeSwapConfig>,
}

pub struct App<P, S>
where
    P: PubsubClient,
    S: Signer,
{
    engine: Engine<P, S, Box<dyn TopTxMonitor>>,
}

impl App<Ws, Wallet<SigningKey>> {
    pub async fn from_config(
        config: Config,
        accounts_dir: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        let client = Arc::new(Provider::new(Ws::connect(config.engine.wss).await?));
        info!("web socket created");

        {
            let network_id = client.get_net_version().await?;
            info!(network_id);
            register_counter!("sandwitch_info", "network_id" => network_id).absolute(1);
        }

        let accounts_dir = accounts_dir.as_ref();
        let keys = Self::read_keys(accounts_dir).await?;
        if keys.is_empty() {
            warn!("no keys found in {}", accounts_dir.display());
        } else {
            info!(
                count = keys.len(),
                "keys collected, initializing accounts..."
            );
        }

        let accounts = Arc::new(Accounts::from_signers(keys, client.clone()).await?);
        info!(count = accounts.len(), "accounts initialized");

        let monitor = Self::make_monitor(client.clone(), accounts.clone(), config.monitors).await?;

        Ok(Self {
            engine: Engine::new(client, accounts, monitor),
        })
    }

    async fn read_keys(dir: impl AsRef<Path>) -> anyhow::Result<Vec<Wallet<SigningKey>>> {
        ReadDirStream::new(fs::read_dir(dir).await?)
            .and_then(|e| fs::read(e.path()))
            .err_into::<anyhow::Error>()
            .and_then(|k| future::ready(SigningKey::from_bytes(&k).map_err(Into::into)))
            .map_ok(Into::into)
            .try_collect::<Vec<_>>()
            .await
    }
}

impl<P, S> App<P, S>
where
    P: PubsubClient + 'static,
    S: Signer + 'static,
{
    pub async fn run(&mut self, cancel: CancellationToken) -> anyhow::Result<()> {
        self.engine.run(cancel).await
    }

    async fn make_monitor(
        client: impl Into<Arc<Provider<P>>>,
        accounts: impl Into<Arc<Accounts<P, S>>>,
        config: MonitorsConfig,
    ) -> anyhow::Result<Box<dyn TopTxMonitor>> {
        let mut monitors = Self::make_monitors(client, accounts, config).await?;

        if monitors.is_empty() {
            return Err(anyhow!("all monitors are disabled"));
        }
        Ok(if monitors.len() == 1 {
            monitors.remove(0)
        } else {
            Box::new(monitors.map(|_| ()))
        })
    }

    async fn make_monitors(
        client: impl Into<Arc<Provider<P>>>,
        accounts: impl Into<Arc<Accounts<P, S>>>,
        config: MonitorsConfig,
    ) -> anyhow::Result<Vec<Box<dyn TopTxMonitor>>> {
        let client = client.into();
        let futs = FuturesUnordered::<LocalBoxFuture<Result<Box<dyn TopTxMonitor>, _>>>::new();

        #[cfg(feature = "pancake_swap")]
        if let Some(cfg) = config.pancake_swap {
            futs.push(
                PancakeSwap::from_config(client.clone(), accounts, cfg)
                    .map_ok(|m| Box::new(m) as Box<dyn TopTxMonitor>)
                    .boxed_local(),
            );
        }

        futs.try_collect().await
    }
}
