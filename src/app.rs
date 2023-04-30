use core::time::Duration;

use std::sync::Arc;

use ethers::{
    core::k256::ecdsa::SigningKey,
    providers::{JsonRpcClient, Middleware, Provider, PubsubClient},
    signers::LocalWallet,
    utils::secret_key_to_address,
};
use futures::{
    future::{self, LocalBoxFuture, TryFutureExt},
    stream::FuturesUnordered,
    try_join, FutureExt, TryStreamExt,
};
use metrics::register_counter;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use sandwitch_engine::{
    monitor::{MultiMonitor, PendingBlockMonitor},
    providers::LatencyProvider,
    Engine, MiddlewareStack,
};

use crate::{providers::timeout::TimeoutProvider, Config, MonitorsConfig};

pub struct App<P, M>
where
    P: JsonRpcClient,
    P::Error: 'static,
    M: PendingBlockMonitor<MiddlewareStack<TimeoutProvider<P>>>,
{
    engine: Engine<TimeoutProvider<P>, M>,
}

const CLIENT_TIMEOUT: Duration = Duration::from_secs(5);

impl<P> App<P, Box<dyn PendingBlockMonitor<MiddlewareStack<TimeoutProvider<P>>>>>
where
    P: PubsubClient + 'static,
{
    pub async fn new(
        client: P,
        signing_key: impl Into<Option<SigningKey>>,
        cfg: Config,
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
    ) -> anyhow::Result<Box<dyn PendingBlockMonitor<MiddlewareStack<TimeoutProvider<P>>>>> {
        let monitors = Self::make_monitors(client, config).await?;
        Ok(match monitors.len() {
            0 => {
                warn!("all monitors are disabled, starting is watch mode...");
                Box::new(())
            }
            1 => monitors.into_iter().next().unwrap(),
            _ => Box::new(monitors),
        })
    }

    #[allow(unused_variables)]
    async fn make_monitors(
        client: impl Into<Arc<MiddlewareStack<TimeoutProvider<P>>>>,
        cfg: MonitorsConfig,
    ) -> anyhow::Result<
        MultiMonitor<Box<dyn PendingBlockMonitor<MiddlewareStack<TimeoutProvider<P>>>>>,
    > {
        let client = client.into();
        let ms = FuturesUnordered::<LocalBoxFuture<_>>::new();

        #[cfg(feature = "tx_logger")]
        if cfg.tx_logger.enabled {
            ms.push(
                future::ok(Box::new(sandwitch_engine::monitor::TxMonitor::from(
                    sandwitch_monitor_logger::LogMonitor,
                )) as Box<dyn PendingBlockMonitor<_>>)
                .boxed_local(),
            );
        }

        // #[cfg(feature = "pancake_swap")]
        // if let Some(cfg) = config.pancake_swap {
        //     tx_monitors.push(
        //         PancakeSwap::from_config(client.clone(), accounts, cfg)
        //             .map_ok(|m| Box::new(m) as Box<dyn TopTxMonitor>)
        //             .boxed_local(),
        //     );
        // }

        ms.try_collect().await
    }
}

impl<P, M> App<P, M>
where
    P: PubsubClient + 'static,
    M: PendingBlockMonitor<MiddlewareStack<TimeoutProvider<P>>>,
{
    pub async fn run(self, cancel: CancellationToken) -> anyhow::Result<()> {
        self.engine.run(cancel).await
    }
}
