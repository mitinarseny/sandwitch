use futures::future::BoxFuture;
use futures::stream::{BoxStream, FuturesUnordered};
use futures::{Stream, StreamExt, TryStreamExt};
use std::future;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

pub mod logger;
pub mod pancake_swap;

pub trait Monitor {
    type Item;
    fn process(self: Box<Self>, stream: BoxStream<'_, Self::Item>) -> BoxFuture<'_, ()>;
}

pub struct MultiMonitor<T> {
    monitors: Vec<Box<dyn Monitor<Item = T> + Send + Sync>>,
}

impl<T: Clone> MultiMonitor<T> {
    pub fn new(monitors: Vec<Box<dyn Monitor<Item = T> + Send + Sync>>) -> Box<Self> {
        Box::new(Self { monitors })
    }
}

impl<T> MultiMonitor<T>
where
    T: Clone + Send + 'static,
{
    async fn do_process(self: Box<Self>, mut stream: impl Stream<Item = T> + Unpin) {
        let (tx, rx) = broadcast::channel(1);

        let monitors: FuturesUnordered<_> = (*self)
            .monitors
            .into_iter()
            .map(|m| {
                tokio::spawn({
                    let s =
                        BroadcastStream::new(tx.subscribe()).filter_map(|r| future::ready(r.ok())); // log
                    m.process(Box::pin(s))
                })
            })
            .collect();

        println!("started monitors");

        while let Some(v) = stream.next().await {
            tx.send(v); // TODO: unwrap
        }
        drop(tx);

        monitors.try_collect::<Vec<()>>(); // TODO: join error?
    }
}

impl<T> Monitor for MultiMonitor<T>
where
    T: Clone + Send + 'static,
{
    type Item = T;

    fn process(self: Box<Self>, stream: BoxStream<'_, Self::Item>) -> BoxFuture<'_, ()> {
        Box::pin(self.do_process(stream))
    }
}
