use std::future;
use std::sync::Arc;

use futures::future::BoxFuture;
use futures::stream::{BoxStream, FuturesUnordered};
use futures::{FutureExt, StreamExt, TryFutureExt, TryStreamExt};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

pub mod logger;
pub mod pancake_swap;

pub trait Monitor<Item> {
    type Error;

    fn process(self: Arc<Self>, item: Item) -> BoxFuture<'static, Result<(), Self::Error>>;
}

pub struct MultiMonitor<Item, E> {
    monitors: Vec<Arc<dyn Monitor<Item, Error = E> + Send + Sync>>,
}

impl<Item, E> MultiMonitor<Item, E> {
    pub fn new(monitors: Vec<Arc<dyn Monitor<Item, Error = E> + Send + Sync>>) -> Self {
        Self { monitors }
    }
}

pub enum MultiError<E> {
    JoinError(tokio::task::JoinError),
    Monitor(E),
}

impl<Item, E> Monitor<Item> for MultiMonitor<Item, E>
where
    Item: Clone + Send + 'static,
    E: Send + 'static,
{
    type Error = MultiError<E>;

    fn process(self: Arc<Self>, item: Item) -> BoxFuture<'static, Result<(), Self::Error>> {
        self.clone().monitors
            .iter()
            .map(|m| tokio::spawn(m.clone().process(item.clone()).map_err(|err| MultiError::Monitor(err))))
            .collect::<FuturesUnordered<_>>()
            .map_err(|err| MultiError::JoinError(err))
            .try_collect::<Vec<Result<(), MultiError<E>>>>()
            .map_ok(|_| ())
            .boxed()
    }
}
