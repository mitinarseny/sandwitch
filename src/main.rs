#![feature(result_option_inspect, result_flattening)]
use std::path::PathBuf;

use anyhow::Context;
use clap::{Args, Parser, ValueHint};
use opentelemetry::{sdk::Resource, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use tokio::{fs, main, signal::ctrl_c};
use tokio_util::sync::CancellationToken;
use tracing::{info, Level, Subscriber};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{filter::Targets, prelude::*, Layer, Registry};

use sandwitch::Config;

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

    #[command(flatten)]
    logging: LoggingArgs,
}

#[derive(Args)]
struct LoggingArgs {
    #[arg(long, value_name = "HOST:PORT")]
    /// Endpoint for OTLP metrics
    otlp_endpoint: Option<String>,
    #[arg(
        short, long,
        action = clap::ArgAction::Count,
    )]
    /// Increase verbosity (error (deafult) -> warn -> info -> debug -> trace)
    verbose: u8,
}

impl LoggingArgs {
    pub fn make_subscriber(self) -> anyhow::Result<impl Subscriber> {
        Ok(Registry::default()
            .with(
                tracing_subscriber::fmt::layer()
                    .map_event_format(|f| f.pretty().with_source_location(false))
                    .with_filter(Targets::from_iter([(
                        env!("CARGO_PKG_NAME"),
                        [
                            Level::ERROR,
                            Level::WARN,
                            Level::INFO,
                            Level::DEBUG,
                            Level::TRACE,
                        ][(self.verbose.min(4)) as usize],
                    )])),
            )
            .with(if let Some(endpoint) = self.otlp_endpoint {
                Some(
                    OpenTelemetryLayer::new(
                        opentelemetry_otlp::new_pipeline()
                            .tracing()
                            .with_exporter(
                                opentelemetry_otlp::new_exporter()
                                    .tonic()
                                    .with_endpoint(endpoint),
                            )
                            .with_trace_config(opentelemetry::sdk::trace::config().with_resource(
                                Resource::new([KeyValue::new(
                                    opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                                    env!("CARGO_PKG_NAME"),
                                )]),
                            ))
                            .install_batch(opentelemetry::runtime::Tokio)?,
                    )
                    .with_filter(Targets::from_iter([
                        (env!("CARGO_PKG_NAME"), Level::TRACE),
                        ("ethers_providers::rpc::provider", Level::TRACE),
                    ])),
                )
            } else {
                None
            }))
    }
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

    tracing::subscriber::set_global_default(args.logging.make_subscriber()?)?;

    let app = config.init(args.keystore_password).await?;

    let cancel = make_ctrl_c_cancel();

    app.run(cancel.child_token()).await?;

    info!("shutdown");
    Ok(())
}

// async fn install_prometheus_metrics_recoder_and_exporter(
//     host: impl AsRef<str>,
//     port: u16,
// ) -> anyhow::Result<()> {
//     let host = host.as_ref();
//     PrometheusBuilder::new()
//         .with_http_listener(
//             net::lookup_host((host, port))
//                 .await
//                 .with_context(|| format!("failed to lookup host: {host}:{port}",))?
//                 .next()
//                 .unwrap(),
//         )
//         .set_buckets_for_metric(
//             Matcher::Suffix("_duration".to_string()),
//             &[
//                 0.01, 0.05, 0.075, 0.1, 0.125, 0.15, 0.175, 0.2, 0.225, 0.25, 0.275, 0.3, 0.35,
//                 0.4, 0.5, 0.6, 0.7, 0.8, 1.,
//             ],
//         )
//         .unwrap()
//         .install()?;
//     register_counter!("sandwitch_build_info", "version" => env!("CARGO_PKG_VERSION")).absolute(1);
//     Ok(())
// }

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
