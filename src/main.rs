#![feature(result_option_inspect, result_flattening)]
use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, ValueHint};
use metrics::register_counter;
use metrics_exporter_prometheus::PrometheusBuilder;
use tokio::{fs, main, net, signal::ctrl_c};
use tokio_util::sync::CancellationToken;
use tracing::{info, metadata::LevelFilter};
use tracing_subscriber::{prelude::*, Registry};

use sandwitch::{App, Config};

#[derive(Parser)]
#[command(version)]
struct Args {
    #[clap(
        short, long,
        value_parser,
        value_hint = ValueHint::FilePath,
        value_name = "FILE",
        default_value_os_t = PathBuf::from("./sandwitch.toml"),
    )]
    config: PathBuf,

    #[arg(
        short,
        long,
        value_parser,
        value_hint = ValueHint::DirPath,
        value_name = "FILE",
        default_value_os_t = PathBuf::from("./accounts"),
    )]
    accounts_dir: PathBuf,

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

#[main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

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

    tracing::subscriber::set_global_default(
        Registry::default().with(
            tracing_subscriber::fmt::layer().with_filter(
                tracing_subscriber::EnvFilter::new(
                    "h2=info,hyper=info,tokio_util=info,ethers_providers=info",
                )
                .add_directive(
                    [
                        LevelFilter::ERROR,
                        LevelFilter::WARN,
                        LevelFilter::INFO,
                        LevelFilter::DEBUG,
                        LevelFilter::TRACE,
                    ][(args.verbose.min(4)) as usize]
                        .into(),
                ),
            ),
        ),
    )?;

    PrometheusBuilder::new()
        .with_http_listener(
            net::lookup_host((args.metrics_host.as_ref(), args.metrics_port))
                .await
                .with_context(|| {
                    format!(
                        "failed to lookup host: {}:{}",
                        args.metrics_host, args.metrics_port
                    )
                })?
                .next()
                .unwrap(),
        )
        .set_buckets(&[
            0.01, 0.05, 0.075, 0.1, 0.125, 0.15, 0.175, 0.2, 0.225, 0.25, 0.275, 0.3, 0.35, 0.4,
            0.5, 0.6, 0.7, 0.8, 1.,
        ])
        .unwrap()
        .install()
        .with_context(|| "unable to install prometheus metrics recorder/exporter")?;
    register_counter!("sandwitch_build_info", "version" => env!("CARGO_PKG_VERSION")).absolute(1);

    let mut app = App::from_config(config, args.accounts_dir).await?;

    let cancel = make_ctrl_c_cancel();
    app.run(cancel.child_token()).await?;

    info!("shutdown");
    Ok(())
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
