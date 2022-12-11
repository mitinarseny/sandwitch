#![feature(result_option_inspect, result_flattening)]
use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, ValueHint};
use metrics::register_counter;
use metrics_exporter_prometheus::PrometheusBuilder;
use tokio::{fs, main, signal::ctrl_c};
use tokio_util::sync::CancellationToken;
use tracing::{info, metadata::LevelFilter};
use tracing_subscriber::{prelude::*, Registry};

use sandwitch::{App, Config};

#[derive(Parser)]
#[clap(version)]
struct Args {
    #[clap(
        default_value_os_t = PathBuf::from("./sandwitch.toml"),
        short, long,
        value_parser,
        value_hint = ValueHint::FilePath,
        value_name = "FILE",
    )]
    config: PathBuf,

    #[clap(
        default_value_os_t = PathBuf::from("./accounts"),
        short,
        long,
        value_parser,
        value_hint = ValueHint::FilePath,
        value_name = "FILE",
    )]
    accounts_dir: PathBuf,

    /// Increase verbosity (error (deafult) -> warn -> info -> debug -> trace)
    #[clap(short, long, action = clap::ArgAction::Count)]
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
