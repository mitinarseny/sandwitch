use std::future;

use futures::future::{try_join, BoxFuture};
use futures::stream::{BoxStream, FuturesUnordered};
use futures::{FutureExt, StreamExt, TryFutureExt, TryStreamExt};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

pub mod pancake_swap;

pub trait Monitor<Item> {
    type Error;

    fn process<'a>(
        &'a mut self,
        stream: BoxStream<'a, Item>,
    ) -> BoxFuture<'a, Result<(), Self::Error>>;
}

pub struct MultiMonitor<Item, E> {
    buffer_size: usize,
    monitors: Vec<Box<dyn Monitor<Item, Error = E>>>,
}

impl<Item, E> MultiMonitor<Item, E> {
    pub fn new(buffer_size: usize, monitors: Vec<Box<dyn Monitor<Item, Error = E>>>) -> Self {
        Self {
            buffer_size,
            monitors,
        }
    }
}

impl<Item, E> Monitor<Item> for MultiMonitor<Item, E>
where
    Item: Clone + Send + 'static,
    E: Send + 'static,
{
    type Error = E;

    fn process<'a>(
        &'a mut self,
        mut stream: BoxStream<'a, Item>,
    ) -> BoxFuture<'a, Result<(), Self::Error>> {
        let (tx, _) = broadcast::channel(self.buffer_size);

        let monitors = self
            .monitors
            .iter_mut()
            .map(|m| {
                m.process(
                    BroadcastStream::new(tx.subscribe())
                        .filter_map(|r| future::ready(r.ok())) // TODO: err?
                        .boxed(),
                )
            })
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>()
            .map_ok(|_| ())
            .boxed();

        try_join(
            async move {
                while let Some(v) = stream.next().await {
                    if tx.send(v).is_err() {
                        break;
                    }
                }
                Ok(())
            },
            monitors,
        )
        .map_ok(|_| ())
        .boxed()
    }
}
