use async_broadcast::broadcast;
use futures::stream::{select_all, BoxStream, StreamExt};

pub mod pancake_swap;

pub trait Monitor<Input> {
    type Output;

    fn process<'a>(&'a mut self, stream: BoxStream<'a, Input>) -> BoxStream<'a, Self::Output>;
}

pub struct MultiMonitor<In, Out> {
    buffer_size: usize,
    monitors: Vec<Box<dyn Monitor<In, Output = Out>>>,
}

impl<In, Out> MultiMonitor<In, Out> {
    pub fn new(buffer_size: usize, monitors: Vec<Box<dyn Monitor<In, Output = Out>>>) -> Self {
        Self {
            buffer_size,
            monitors,
        }
    }
}

impl<In, Out> Monitor<In> for MultiMonitor<In, Out>
where
    In: Clone + Send + Sync,
{
    type Output = Out;

    fn process<'a>(&'a mut self, mut stream: BoxStream<'a, In>) -> BoxStream<'a, Self::Output> {
        let (tx, rx) = broadcast(self.buffer_size);

        select_all(
            self.monitors
                .iter_mut()
                .map(|m| m.process(rx.clone().boxed())),
        )
        .take_until(async move {
            while let Some(v) = stream.next().await {
                if !tx.broadcast(v).await.is_ok() {
                    break;
                }
            }
        })
        .boxed()
    }
}
