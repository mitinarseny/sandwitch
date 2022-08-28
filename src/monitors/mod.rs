use futures::future::BoxFuture;
use futures::stream::{BoxStream, FuturesUnordered};
use futures::{FutureExt, Stream, StreamExt, TryStreamExt};
use std::convert::Infallible;
use std::future;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

pub mod logger;
pub mod pancake_swap;

pub trait Monitor {
    type Item;
    type Error;

    fn process(
        self: Box<Self>,
        stream: BoxStream<'_, Self::Item>,
    ) -> BoxFuture<'_, Result<(), Self::Error>>;
}

pub struct MultiMonitor<T, E> {
    monitors: Vec<Box<dyn Monitor<Item = T, Error = E> + Send + Sync>>,
}

impl<T: Clone, E> MultiMonitor<T, E> {
    pub fn new(monitors: Vec<Box<dyn Monitor<Item = T, Error = E> + Send + Sync>>) -> Box<Self> {
        Box::new(Self { monitors })
    }
}

impl<T, E> Monitor for MultiMonitor<T, E>
where
    T: Clone + Send + 'static,
    E: Clone + Send + 'static,
{
    type Item = T;
    type Error = E;

    fn process(
        self: Box<Self>,
        mut stream: BoxStream<'_, Self::Item>,
    ) -> BoxFuture<'_, Result<(), Self::Error>> {
        Box::pin(async move {
            let (tx, _) = broadcast::channel(1);

            let monitors: FuturesUnordered<_> = (*self)
                .monitors
                .into_iter()
                .map(|m| {
                    tokio::spawn({
                        let stream = BroadcastStream::new(tx.subscribe())
                            .filter_map(|r| future::ready(r.ok())); // log
                        m.process(Box::pin(stream))
                    })
                })
                .collect();

            println!("started monitors");

            // stream.map(|r| Ok(r)).forward(&mut tx);

            while let Some(v) = stream.next().await {
                tx.send(v); // TODO: unwrap
            }

            monitors.try_collect::<Vec<Result<(), E>>>().await; // TODO: join error?
            Ok(())
        })
    }
}
