#![feature(result_option_inspect)]
use std::path::PathBuf;

use anyhow::Context;
use metrics::register_counter;
use metrics_exporter_prometheus::PrometheusBuilder;
use sandwitch::*;

use clap::{Parser, ValueHint};
use tokio::{fs, main};
use tracing::info;
use tracing::metadata::LevelFilter;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;

#[derive(Parser)]
#[clap(version)]
struct Args {
    #[clap(default_value_os_t = PathBuf::from("./sandwitch.toml"), short, long, value_parser, value_hint = ValueHint::FilePath, value_name = "FILE")]
    config: PathBuf,

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
                tracing_subscriber::EnvFilter::new("h2=info,hyper=info,tokio_util=info")
                    .add_directive(
                        [
                            LevelFilter::ERROR,
                            LevelFilter::WARN,
                            LevelFilter::INFO,
                            LevelFilter::DEBUG,
                            LevelFilter::TRACE,
                        ][(args.verbose % 5) as usize]
                            .into(),
                    ),
            ),
        ),
    )?;

    PrometheusBuilder::new()
        .install()
        .with_context(|| "unable to install prometheus metrics recorder/exporter")?;
    register_counter!("sandwitch_build_info", "version" => env!("CARGO_PKG_VERSION")).absolute(1);

    info!("initializing");
    let mut app = App::from_config(config).await?;

    info!("run");
    app.run().await
}
