use serde::Deserialize;
use url::Url;

use sandwitch_engine::config::Config as EngineConfig;

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
    // #[cfg(feature = "pancake_swap")]
    // pub pancake_swap: Option<PancakeSwapConfig>,
}
