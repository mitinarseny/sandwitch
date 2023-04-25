#![feature(result_option_inspect, result_flattening)]
use std::path::PathBuf;

use anyhow::{anyhow, Context};
use clap::{Parser, ValueHint};
use ethers::core::k256::ecdsa::SigningKey;
use impl_tools::autoimpl;
use metrics::register_counter;
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder};
use serde::Deserialize;
use tokio::{fs, main, net, signal::ctrl_c};
use tokio_util::sync::CancellationToken;
use tracing::{info, metadata::LevelFilter};
use tracing_subscriber::{prelude::*, Registry};
use url::Url;

use sandwitch::{App, Config as AppConfig};

#[derive(Parser)]
#[command(version)]
struct CliArgs {
    #[arg(
        short, long,
        value_parser,
        value_hint = ValueHint::FilePath,
        value_name = "FILE",
        default_value_os_t = PathBuf::from("./sandwitch.toml"),
    )]
    config: PathBuf,

    /// Password to decrypt keystore
    #[arg(long, env = "SANDWITCH_KEYSTORE_PASSWORD")]
    keystore_password: Option<String>,

    #[arg(
        long,
        value_hint = ValueHint::Hostname,
        value_name = "HOST",
        default_value_t = String::from("127.0.0.1"),
    )]
    /// Host for Prometheus metrics
    metrics_host: String,

    #[arg(
        long,
        value_parser = clap::value_parser!(u16).range(1..),
        value_name = "PORT",
        default_value_t = 9000,
    )]
    /// Port for Prometheus metrics
    metrics_port: u16,

    #[arg(
        short, long,
        action = clap::ArgAction::Count,
    )]
    /// Increase verbosity (error (deafult) -> warn -> info -> debug -> trace)
    verbose: u8,
}

#[derive(Deserialize)]
#[autoimpl(Deref using self.app)]
pub struct Config {
    #[serde(flatten)]
    pub app: AppConfig,
    pub keystore: Option<KeyStore>,
}

#[derive(Deserialize, Debug)]
pub struct KeyStore {
    pub path: PathBuf,
}

#[main]
async fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();

    let config: Config = toml::from_str(
        fs::read_to_string(&args.config)
            .await
            .with_context(|| format!("failed to read config file '{}'", args.config.display()))?
            .as_str(),
    )
    .with_context(|| {
        format!(
            "failed to parse TOML config file '{}'",
            args.config.display()
        )
    })?;

    init_tracing_subscriber(
        [
            LevelFilter::ERROR,
            LevelFilter::WARN,
            LevelFilter::INFO,
            LevelFilter::DEBUG,
            LevelFilter::TRACE,
        ][(args.verbose.min(4)) as usize],
    )
    .with_context(|| "failed to init tracing subscriber")?;

    install_prometheus_metrics_recoder_and_exporter(args.metrics_host, args.metrics_port)
        .await
        .with_context(|| "failed to install prometheus metrics recorder/exporter")?;

    info!("connecting to node...");
    let client = connect(&config.network.node).await?;
    info!("connected to node");

    let app = App::new(
        client,
        config
            .keystore
            .zip(args.keystore_password)
            .map(|(keystore, keystore_password)| {
                let secret = eth_keystore::decrypt_key(keystore.path, keystore_password)?;
                anyhow::Ok(SigningKey::from_bytes(secret.as_slice().into())?)
            })
            .transpose()?,
        config.app,
    )
    .await?;

    let cancel = make_ctrl_c_cancel();

    app.run(cancel.child_token()).await?;

    info!("shutdown");
    Ok(())
}

fn init_tracing_subscriber(level: LevelFilter) -> anyhow::Result<()> {
    // TODO: opentelemetry, graphana tempo
    tracing::subscriber::set_global_default(
        Registry::default().with(
            tracing_subscriber::fmt::layer()
                .map_event_format(|f| f.pretty().with_source_location(false)) // TODO: remove
                .with_filter(
                    tracing_subscriber::EnvFilter::new(
                        "h2=info,hyper=info,tokio_util=info,ethers_providers=info",
                    )
                    .add_directive(level.into()),
                ),
        ),
    )?;
    Ok(())
}

async fn install_prometheus_metrics_recoder_and_exporter(
    host: impl AsRef<str>,
    port: u16,
) -> anyhow::Result<()> {
    let host = host.as_ref();
    PrometheusBuilder::new()
        .with_http_listener(
            net::lookup_host((host, port))
                .await
                .with_context(|| format!("failed to lookup host: {host}:{port}",))?
                .next()
                .unwrap(),
        )
        .set_buckets_for_metric(
            Matcher::Suffix("_duration".to_string()),
            &[
                0.01, 0.05, 0.075, 0.1, 0.125, 0.15, 0.175, 0.2, 0.225, 0.25, 0.275, 0.3, 0.35,
                0.4, 0.5, 0.6, 0.7, 0.8, 1.,
            ],
        )
        .unwrap()
        .install()?;
    register_counter!("sandwitch_build_info", "version" => env!("CARGO_PKG_VERSION")).absolute(1);
    Ok(())
}

#[cfg(all(feature = "ws", not(feature = "ipc")))]
type connect = connect_ws;
#[cfg(feature = "ws")]
async fn connect_ws(url: &Url) -> anyhow::Result<ethers::providers::Ws> {
    Ok(ethers::providers::Ws::connect(url).await?)
}

#[cfg(all(feature = "ipc", not(feature = "ws")))]
type connect = connect_ipc;
#[cfg(feature = "ipc")]
async fn connect_ipc(url: &Url) -> anyhow::Result<ethers::providers::Ipc> {
    Ok(
        ethers::providers::Ipc::connect(
            url.to_file_path().map_err(|_| anyhow!("invalid IPC url"))?,
        )
        .await?,
    )
}

#[cfg(all(feature = "ipc", feature = "ws"))]
async fn connect(
    url: &Url,
) -> anyhow::Result<sandwitch::providers::OneOf<ethers::providers::Ws, ethers::providers::Ipc>> {
    Ok(match url.scheme() {
        "wss" => sandwitch::providers::OneOf::P1(connect_ws(url).await?),
        "file" => sandwitch::providers::OneOf::P2(connect_ipc(url).await?),
        _ => return Err(anyhow!("invalid node url: {url}")),
    })
}

fn make_ctrl_c_cancel() -> CancellationToken {
    let cancel = CancellationToken::new();
    let child = cancel.child_token();
    tokio::spawn({
        let cancel_guard = cancel.drop_guard();
        async move {
            ctrl_c().await.expect("failed to set CTRL+C handler");
            info!("shutdown requested...");
            drop(cancel_guard);
        }
    });
    child
}
