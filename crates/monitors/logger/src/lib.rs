use futures::{
    future::{self, BoxFuture},
    FutureExt,
};
use tracing::{info, instrument};

use sandwitch_engine::block::TxWithLogs;

use sandwitch_engine::monitor::PendingTxMonitor;

pub struct LogMonitor;

impl PendingTxMonitor for LogMonitor {
    #[instrument(skip_all, fields(?tx.hash, logs_count = tx.logs.len()))]
    fn process_pending_tx<'a>(&'a self, tx: &'a TxWithLogs) -> BoxFuture<'a, anyhow::Result<()>> {
        info!("seen");
        future::ok(()).boxed()
    }
}
