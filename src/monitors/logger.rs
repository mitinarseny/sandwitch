use super::Monitor;
use futures::future::BoxFuture;
use futures::Stream;

pub struct Logger {}

// impl Monitor for Logger {
//     fn process_transactions(
//         &self,
//         txs: Box<dyn Stream<Item = web3::types::Transaction>>,
//     ) -> BoxFuture<'_, ()> {
//         while let Some(tx)
//     }
// }
