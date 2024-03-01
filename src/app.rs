use core::time::Duration;

use std::sync::Arc;

use anyhow::Context;
use ethers::{
    core::k256::ecdsa::SigningKey,
    providers::{JsonRpcClient, Middleware, Provider, PubsubClient},
    signers::LocalWallet,
    utils::secret_key_to_address,
};
use futures::{
    future::{LocalBoxFuture, TryFutureExt},
    stream::FuturesUnordered,
    try_join, FutureExt, TryStreamExt,
};
use metrics::register_counter;
use sandwitch_monitor_erc20::PancakeMonitor;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use sandwitch_engine::{
    monitor::{BlockMonitor, MultiMonitor, NoopMonitor},
    providers::LatencyProvider,
    Engine, MiddlewareStack,
};

use crate::{providers::timeout::TimeoutProvider, AppConfig, MonitorsConfig};

pub struct App<P>
where
    P: JsonRpcClient,
    P::Error: 'static,
{
    engine: Engine<TimeoutProvider<P>, Box<dyn BlockMonitor<MiddlewareStack<TimeoutProvider<P>>>>>,
}

const CLIENT_TIMEOUT: Duration = Duration::from_secs(5);

impl<P> App<P>
where
    P: PubsubClient + 'static,
{
    pub async fn new(
        client: P,
        signing_key: impl Into<Option<SigningKey>>,
        cfg: AppConfig,
    ) -> anyhow::Result<Self> {
        let client = Arc::new(Provider::new(LatencyProvider::new(TimeoutProvider::new(
            client,
            CLIENT_TIMEOUT,
        ))));
        info!("initializing...");
        let (network_id, chain_id, client_version) = try_join!(
            client.get_net_version(),
            client.get_chainid().map_ok(|v| v.as_u64()),
            client.client_version(),
        )?;
        info!(network_id, chain_id, client_version, "node info");
        register_counter!(
            "sandwitch_info",
            "network_id" => network_id,
            "chain_id" => chain_id.to_string(),
            "version" => client_version,
        )
        .absolute(1);

        let monitor = Self::make_monitor(client.clone(), cfg.monitors).await?;

        Ok(Self {
            engine: Engine::new(
                client,
                cfg.engine,
                signing_key.into().map(|signing_key| {
                    let address = secret_key_to_address(&signing_key);
                    LocalWallet::new_with_signer(signing_key, address, chain_id)
                }),
                monitor,
            )
            .await?,
        })
    }

    async fn make_monitor(
        client: impl Into<Arc<MiddlewareStack<TimeoutProvider<P>>>>,
        config: MonitorsConfig,
    ) -> anyhow::Result<Box<dyn BlockMonitor<MiddlewareStack<TimeoutProvider<P>>>>> {
        let monitors = Self::make_monitors(client, config).await?;
        Ok(match monitors.len() {
            0 => {
                warn!("all monitors are disabled, starting is watch mode...");
                Box::new(NoopMonitor)
            }
            1 => monitors.into_iter().next().unwrap(),
            _ => Box::new(monitors),
        })
    }

    #[allow(unused_variables)]
    async fn make_monitors(
        client: impl Into<Arc<MiddlewareStack<TimeoutProvider<P>>>>,
        cfg: MonitorsConfig,
    ) -> anyhow::Result<MultiMonitor<Box<dyn BlockMonitor<MiddlewareStack<TimeoutProvider<P>>>>>>
    {
        let client = client.into();
        let ms = FuturesUnordered::<LocalBoxFuture<_>>::new();

        #[cfg(feature = "tx_logger")]
        if cfg.tx_logger.enabled {
            ms.push(
                future::ok(Box::new(sandwitch_engine::monitor::TxMonitor::from(
                    sandwitch_monitor_logger::LogMonitor,
                )) as Box<dyn BlockMonitor<_>>)
                .boxed_local(),
            );
        }

        #[cfg(feature = "pancake_swap")]
        if let Some(cfg) = cfg.pancake_swap {
            ms.push(
                PancakeMonitor::from_config(client.clone(), cfg)
                    .map_ok(|m| Box::new(m) as Box<dyn BlockMonitor<_>>)
                    .map(|r| r.context("pancake"))
                    .boxed_local(),
            );
        }

        ms.try_collect().await
    }
    pub async fn run(self, cancel: CancellationToken) -> anyhow::Result<()> {
        self.engine.run(cancel).await
    }
}
