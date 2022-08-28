use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use sandwitch::*;

use clap::{Parser, ValueHint};
use tokio::{fs, main};

#[derive(Parser)]
#[clap(version)]
struct Args {
    #[clap(default_value_os_t = PathBuf::from("./sandwitch.toml"), short, long, value_parser, value_hint = ValueHint::FilePath, value_name = "FILE")]
    config: PathBuf,
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

    let app = App::from_config(config).await?;
    Arc::new(app).run().await
}
