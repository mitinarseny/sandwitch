use core::time::Duration;

use ethers::types::Address;
use serde::Deserialize;
use serde_with::{serde_as, DurationMilliSeconds};

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(rename = "block_interval_ms")]
    #[serde_as(as = "DurationMilliSeconds")]
    pub block_interval: Duration,

    #[serde(rename = "tx_propagation_delay_ms")]
    #[serde_as(as = "DurationMilliSeconds")]
    pub tx_propagation_delay: Duration,

    pub multicall: Address,
}
