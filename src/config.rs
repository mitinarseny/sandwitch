use std::path::PathBuf;

use anyhow::anyhow;
use ethers::{prelude::k256::ecdsa::SigningKey, providers::PubsubClient};
use impl_tools::autoimpl;
use sandwitch_monitor_erc20::PancakeConfig;
use serde::Deserialize;
use tracing::info;
use url::Url;

use sandwitch_engine::config::Config as EngineConfig;

use crate::{providers::one_of::OneOf, App};

#[derive(Deserialize)]
#[autoimpl(Deref using self.app)]
pub struct Config {
    #[serde(flatten)]
    pub app: AppConfig,
    pub keystore: Option<KeyStore>,
}

impl Config {
    pub async fn init(
        self,
        keystore_password: impl Into<Option<String>>,
    ) -> anyhow::Result<App<impl PubsubClient>> {
        App::new(
            {
                info!("connecting to node...");
                let client = self.network.connect().await?;
                info!("connected to node");
                client
            },
            self.keystore
                .zip(keystore_password.into())
                .map(|(keystore, keystore_password)| {
                    let secret = eth_keystore::decrypt_key(keystore.path, keystore_password)?;
                    anyhow::Ok(SigningKey::from_bytes(secret.as_slice().into())?)
                })
                .transpose()?,
            self.app,
        )
        .await
    }
}

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub network: NetworkConfig,
    pub engine: EngineConfig,
    pub monitors: MonitorsConfig,
}

#[derive(Deserialize, Debug)]
pub struct KeyStore {
    pub path: PathBuf,
}

#[derive(Deserialize, Debug)]
pub struct NetworkConfig {
    pub node: Url,
}

impl NetworkConfig {
    #[cfg(all(feature = "ipc", not(feature = "ws")))]
    pub async fn connect(&self) -> anyhow::Result<impl PubsubClient> {
        self.connect_ipc().await
    }

    #[cfg(all(feature = "ws", not(feature = "ipc")))]
    pub async fn connect(&self) -> anyhow::Result<impl PubsubClient> {
        self.connect_ws().await
    }

    #[cfg(all(feature = "ws", feature = "ipc"))]
    pub async fn connect(&self) -> anyhow::Result<impl PubsubClient> {
        Ok(match self.node.scheme() {
            "ws" | "wss" => OneOf::P1(self.connect_ws().await?),
            "file" => OneOf::P2(self.connect_ipc().await?),
            _ => return Err(anyhow!("invalid node url: {}", self.node)),
        })
    }

    #[cfg(feature = "ipc")]
    async fn connect_ipc(&self) -> anyhow::Result<impl PubsubClient> {
        ethers::providers::Ipc::connect(
            self.node
                .to_file_path()
                .map_err(|_| anyhow!("invalid IPC url"))?,
        )
        .await
        .map_err(Into::into)
    }

    #[cfg(feature = "ws")]
    async fn connect_ws(&self) -> anyhow::Result<impl PubsubClient> {
        ethers::providers::Ws::connect(&self.node)
            .await
            .map_err(Into::into)
    }
}

#[derive(Deserialize, Debug)]
pub struct MonitorConfig<C> {
    pub enabled: bool,
    #[serde(flatten)]
    pub cfg: C,
}

#[derive(Deserialize, Debug)]
pub struct MonitorsConfig {
    #[cfg(feature = "tx_logger")]
    pub tx_logger: MonitorConfig<()>,
    // pub tx_logger:
    #[cfg(feature = "pancake_swap")]
    pub pancake_swap: Option<PancakeConfig>,
}
